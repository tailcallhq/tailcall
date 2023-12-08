use std::collections::BTreeSet;
use std::fs;
use std::sync::Arc;

use anyhow::Result;
use async_graphql::http::GraphiQLSource;
use async_graphql::ServerError;
use hyper::{Body, HeaderMap, Request, Response, StatusCode};
use protobuf::reflect::FileDescriptor;
use serde::de::DeserializeOwned;
use serde::Serialize;

use super::request_context::RequestContext;
use super::ServerContext;
use crate::async_graphql_hyper::{GraphQLRequestLike, GraphQLResponse};

fn graphiql() -> Result<Response<Body>> {
  Ok(Response::new(Body::from(
    GraphiQLSource::build()
      .title("Tailcall - GraphQL IDE")
      .endpoint("/graphql")
      .finish(),
  )))
}

fn not_found() -> Result<Response<Body>> {
  Ok(Response::builder().status(StatusCode::NOT_FOUND).body(Body::empty())?)
}

fn create_request_context(req: &Request<Body>, server_ctx: &ServerContext) -> RequestContext {
  let upstream = server_ctx.blueprint.upstream.clone();
  let allowed = upstream.get_allowed_headers();
  let headers = create_allowed_headers(req.headers(), &allowed);
  RequestContext::from(server_ctx).req_headers(headers)
}

fn update_cache_control_header(
  response: GraphQLResponse,
  server_ctx: &ServerContext,
  req_ctx: Arc<RequestContext>,
) -> GraphQLResponse {
  if server_ctx.blueprint.server.enable_cache_control_header {
    let ttl = req_ctx.get_min_max_age().unwrap_or(0);
    let cache_public_flag = req_ctx.is_cache_public().unwrap_or(true);
    return response.set_cache_control(ttl, cache_public_flag);
  }
  response
}

pub fn update_response_headers(resp: &mut hyper::Response<hyper::Body>, server_ctx: &ServerContext) {
  if !server_ctx.blueprint.server.response_headers.is_empty() {
    resp
      .headers_mut()
      .extend(server_ctx.blueprint.server.response_headers.clone());
  }
}

pub async fn test(_req: Request<Body>, _server_ctx: &ServerContext) -> Result<Response<Body>> {
  let client = reqwest::Client::builder().http2_prior_knowledge().build()?;
  let proto = r#"syntax = "proto3";

message News {
    string id = 1;
    string title = 2;
    string body = 3;
    string postImage = 4;
}

service NewsService {
    rpc GetAllNews (Empty) returns (NewsList) {}
    rpc GetNews (NewsId) returns (News) {}
    rpc DeleteNews (NewsId) returns (Empty) {}
    rpc EditNews (News) returns (News) {}
    rpc AddNews (News) returns (News) {}
}

message NewsId {
    string id = 1;
}

message Empty {}

message NewsList {
   repeated News news = 1;
}"#;

  let temp_dir = tempfile::tempdir().unwrap();
  let tempfile = temp_dir.path().join("news.proto");
  // For now we need to write files to the disk.
  fs::write(&tempfile, proto).unwrap();

  // Parse text `.proto` file to `FileDescriptorProto` message.
  // Note this API is not stable and subject to change.
  // But binary protos can always be generated manually with `protoc` command.
  let file_descriptor_protos = protobuf_parse::Parser::new()
    .pure()
    .includes(&[temp_dir.path().to_path_buf()])
    .input(&tempfile)
    .parse_and_typecheck()?;

  println!("{:?}", file_descriptor_protos.file_descriptors.len());

  let file_descriptor_proto = file_descriptor_protos
    .file_descriptors
    .into_iter()
    .next()
    .expect("file descriptor proto");
  let file_descriptor = FileDescriptor::new_dynamic(file_descriptor_proto, &[])?;

  // Find the message descriptor for 'NewsList'.
  let news_list_descriptor = file_descriptor
    .message_by_package_relative_name("NewsList")
    .expect("message descriptor");
  println!("{:?}", news_list_descriptor);

  let mut headers = HeaderMap::new();
  #[derive(Serialize, Default)]
  struct Empty {}
  headers.insert(
    reqwest::header::CONTENT_TYPE,
    reqwest::header::HeaderValue::from_static("application/grpc"),
  );

  let response = client
    .get("http://localhost:50051/NewsService/GetAllNews")
    .headers(headers)
    .body(serde_json::to_vec(&Empty {})?)
    .send()
    .await?;
  if response.status().is_success() {
    let bytes = response.bytes().await?;
    let news_list_message = news_list_descriptor.parse_from_bytes(&bytes[5..]);
    let news_list_json = protobuf::text_format::print_to_string(&*news_list_message?); // Todo convert to json
    Ok(Response::new(Body::from(news_list_json)))
  } else {
    todo!()
  }
}

pub async fn graphql_request<T: DeserializeOwned + GraphQLRequestLike>(
  req: Request<Body>,
  server_ctx: &ServerContext,
) -> Result<Response<Body>> {
  let req_ctx = Arc::new(create_request_context(&req, server_ctx));
  let bytes = hyper::body::to_bytes(req.into_body()).await?;
  let request = serde_json::from_slice::<T>(&bytes);
  match request {
    Ok(request) => {
      let mut response = request.data(req_ctx.clone()).execute(&server_ctx.schema).await;
      response = update_cache_control_header(response, server_ctx, req_ctx);
      let mut resp = response.to_response()?;
      update_response_headers(&mut resp, server_ctx);
      Ok(resp)
    }
    Err(err) => {
      log::error!(
        "Failed to parse request: {}",
        String::from_utf8(bytes.to_vec()).unwrap()
      );

      let mut response = async_graphql::Response::default();
      let server_error = ServerError::new(format!("Unexpected GraphQL Request: {}", err), None);
      response.errors = vec![server_error];

      Ok(GraphQLResponse::from(response).to_response()?)
    }
  }
}

fn create_allowed_headers(headers: &HeaderMap, allowed: &BTreeSet<String>) -> HeaderMap {
  let mut new_headers = HeaderMap::new();
  for (k, v) in headers.iter() {
    if allowed.contains(k.as_str()) {
      new_headers.insert(k, v.clone());
    }
  }

  new_headers
}

pub async fn handle_request<T: DeserializeOwned + GraphQLRequestLike>(
  req: Request<Body>,
  state: Arc<ServerContext>,
) -> Result<Response<Body>> {
  match *req.method() {
    hyper::Method::GET if req.uri().path() == "/test" => test(req, state.as_ref()).await,
    hyper::Method::POST if req.uri().path() == "/graphql" => graphql_request::<T>(req, state.as_ref()).await,
    hyper::Method::GET if state.blueprint.server.enable_graphiql => graphiql(),
    _ => not_found(),
  }
}
