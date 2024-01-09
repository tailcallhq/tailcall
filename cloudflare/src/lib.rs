use std::sync::{Arc, RwLock};

use lazy_static::lazy_static;
use tailcall::async_graphql_hyper::GraphQLRequest;
use tailcall::blueprint::Blueprint;
use tailcall::config::reader::ConfigReader;
use tailcall::config::{Config, Upstream};
use tailcall::http::{handle_request, HttpClientOptions, ServerContext};
use tailcall::io::file::FileIO;
use worker::*;

lazy_static! {
  static ref SERV_CTX: RwLock<Option<Arc<ServerContext>>> = RwLock::new(None);
}

async fn make_req(file: impl FileIO) -> Result<Config> {
  let http_client = tailcall::io::http::init_http_cloudflare(&Upstream::default(), &HttpClientOptions::default());
  let reader = ConfigReader::init(file);
  reader
    .read(
      &[
        "https://raw.githubusercontent.com/tailcallhq/tailcall/main/examples/jsonplaceholder.graphql".to_string(), // add/edit the SDL links to this list
      ],
      http_client,
    )
    .await
    .map_err(conv_err)
}

#[event(fetch)]
async fn main(req: Request, _: Env, _: Context) -> Result<Response> {
  let mut server_ctx = get_option().await;
  if server_ctx.is_none() {
    let cfg = make_req(tailcall::io::file::init_cloudflare()).await.map_err(conv_err);
    let cfg = match cfg {
      Ok(cfg) => cfg,
      Err(e) => {
        return Response::ok(format!("cfg err: {}", e.to_string()));
      }
    };
    let blueprint = Blueprint::try_from(&cfg).map_err(conv_err)?;
    let universal_http_client = Arc::new(tailcall::io::http::init_http_cloudflare(
      &blueprint.upstream,
      &HttpClientOptions::default(),
    ));

    let http2_only_client = Arc::new(tailcall::io::http::init_http_cloudflare(
      &blueprint.upstream,
      &HttpClientOptions { http2_only: true },
    ));
    let serv_ctx = Arc::new(ServerContext::new(blueprint, universal_http_client, http2_only_client));
    *SERV_CTX.write().unwrap() = Some(serv_ctx.clone());
    server_ctx = Some(serv_ctx);
  }
  let resp = handle_request::<GraphQLRequest>(
    convert_to_hyper_request(req).await?,
    server_ctx.ok_or(Error::from("Unable to initiate connection"))?.clone(),
  )
  .await
  .map_err(conv_err)?;
  let resp = make_request(resp).await.map_err(conv_err)?;
  Ok(resp)
}

async fn get_option() -> Option<Arc<ServerContext>> {
  SERV_CTX.read().unwrap().clone()
}

async fn make_request(response: hyper::Response<hyper::Body>) -> Result<Response> {
  let buf = hyper::body::to_bytes(response).await.map_err(conv_err)?;
  let text = std::str::from_utf8(&buf).map_err(conv_err)?;
  let mut response = Response::ok(text).map_err(conv_err)?;
  response
    .headers_mut()
    .append("Content-Type", "text/html")
    .map_err(conv_err)?;
  Ok(response)
}

fn convert_method(worker_method: Method) -> hyper::Method {
  let method_str = &*worker_method.to_string().to_uppercase();

  match method_str {
    "GET" => Ok(hyper::Method::GET),
    "POST" => Ok(hyper::Method::POST),
    "PUT" => Ok(hyper::Method::PUT),
    "DELETE" => Ok(hyper::Method::DELETE),
    "HEAD" => Ok(hyper::Method::HEAD),
    "OPTIONS" => Ok(hyper::Method::OPTIONS),
    "PATCH" => Ok(hyper::Method::PATCH),
    "CONNECT" => Ok(hyper::Method::CONNECT),
    "TRACE" => Ok(hyper::Method::TRACE),
    _ => Err("Unsupported HTTP method"),
  }
  .unwrap()
}

async fn convert_to_hyper_request(mut worker_request: Request) -> Result<hyper::Request<hyper::Body>> {
  let body = worker_request.text().await?;
  let method = worker_request.method();
  let uri = worker_request.url()?.as_str().to_string();
  let headers = worker_request.headers();
  let mut builder = hyper::Request::builder().method(convert_method(method)).uri(uri);
  for (k, v) in headers {
    builder = builder.header(k, v);
  }
  builder.body(hyper::body::Body::from(body)).map_err(conv_err)
}

fn conv_err<T: std::fmt::Display>(e: T) -> Error {
  Error::from(format!("{}", e.to_string()))
}
