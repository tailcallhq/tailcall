use std::collections::BTreeSet;
use std::sync::Arc;

use anyhow::Result;
use async_graphql::http::GraphiQLSource;
use client::DefaultHttpClient;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, HeaderMap, Request, Response, StatusCode};
use serde::de::DeserializeOwned;
use serde_json::json;

use super::request_context::RequestContext;
use super::ServerContext;
use crate::async_graphql_hyper::{GraphQLBatchRequest, GraphQLRequest, GraphQLRequestLike, GraphQLResponse};
use crate::blueprint::Blueprint;
use crate::cli::CLIError;
use crate::config::Config;
use crate::http::client;

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
    if let Some(ttl) = req_ctx.get_min_max_age() {
      return response.set_cache_control(ttl as i32);
    }
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
      let mut resp = Response::new(Body::from(
        json!({
          "error": "Unexpected graphQL request",
          "message": err.to_string()
        })
        .to_string(),
      ));
      *resp.status_mut() = StatusCode::BAD_REQUEST;
      Ok(resp)
    }
  }
}

pub async fn graphql_single_request(req: Request<Body>, server_ctx: &ServerContext) -> Result<Response<Body>> {
  graphql_request::<GraphQLRequest>(req, server_ctx).await
}

pub async fn graphql_batch_request(req: Request<Body>, server_ctx: &ServerContext) -> Result<Response<Body>> {
  graphql_request::<GraphQLBatchRequest>(req, server_ctx).await
}

async fn handle_single_request(req: Request<Body>, state: Arc<ServerContext>) -> Result<Response<Body>> {
  match *req.method() {
    hyper::Method::GET if state.blueprint.server.enable_graphiql => graphiql(),
    hyper::Method::POST if req.uri().path() == "/graphql" => graphql_single_request(req, state.as_ref()).await,
    _ => not_found(),
  }
}

async fn handle_batch_request(req: Request<Body>, state: Arc<ServerContext>) -> Result<Response<Body>> {
  match *req.method() {
    hyper::Method::POST if req.uri().path() == "/graphql" => graphql_batch_request(req, state.as_ref()).await,
    hyper::Method::GET if state.blueprint.server.enable_graphiql => graphiql(),
    _ => not_found(),
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

pub async fn start_server(config: Config) -> Result<()> {
  let blueprint = Blueprint::try_from(&config).map_err(CLIError::from)?;
  let http_client = Arc::new(DefaultHttpClient::new(&blueprint.upstream));
  let state = Arc::new(ServerContext::new(blueprint.clone(), http_client));
  let addr = (blueprint.server.hostname, blueprint.server.port).into();
  let state_clone = state.clone();

  let make_svc_single_req = make_service_fn(move |_conn| {
    let state = Arc::clone(&state);
    async move { Ok::<_, anyhow::Error>(service_fn(move |req| handle_single_request(req, state.clone()))) }
  });

  let make_svc_batch_req = make_service_fn(move |_conn| {
    let state = Arc::clone(&state_clone);
    async move { Ok::<_, anyhow::Error>(service_fn(move |req| handle_batch_request(req, state.clone()))) }
  });

  let _ = tokio::runtime::Builder::new_multi_thread()
    .worker_threads(blueprint.server.worker)
    .enable_all()
    .build()
    .unwrap()
    .spawn(async move {
      let builder = hyper::Server::try_bind(&addr).map_err(CLIError::from)?;

      let enable_graphiql = blueprint.server.enable_graphiql;
      log::info!("🚀 Tailcall launched at [{}]", addr);
      if enable_graphiql {
        log::info!("🌍 Playground: http://{}", addr);
      }

      let r = if blueprint.server.enable_batch_requests {
        builder.serve(make_svc_batch_req).await
      } else {
        builder.serve(make_svc_single_req).await
      };

      r.map_err(CLIError::from)
    })
    .await?;

  Ok(())
}
