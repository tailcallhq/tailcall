use std::sync::Arc;

use anyhow::anyhow;
use lazy_static::lazy_static;
use tailcall::async_graphql_hyper::GraphQLRequest;
use tailcall::blueprint::Blueprint;
use tailcall::config::reader::ConfigReader;
use tailcall::config::Config;
use tailcall::http::{handle_request, DefaultHttpClient, ServerContext};
use tokio::sync::RwLock;
use worker::*;

lazy_static! {
  static ref SERV_CTX: RwLock<Option<Arc<ServerContext>>> = RwLock::new(None);
}

#[event(fetch)]
async fn main(req: Request, _: Env, _: Context) -> Result<Response> {
  let mut x = get_option().await;
  if x.is_none() {
    let cfg = make_req().await.map_err(|e| Error::from(format!("{}", e)))?;
    let blueprint = Blueprint::try_from(&cfg).map_err(|e| Error::from(format!("{}", e)))?;
    let http_client = Arc::new(DefaultHttpClient::new(&blueprint.upstream));
    let serv_ctx = Arc::new(ServerContext::new(blueprint, http_client));
    *SERV_CTX.write().await = Some(serv_ctx.clone());
    x = Some(serv_ctx);
  }
  let resp = handle_request::<GraphQLRequest>(
    convert_to_hyper_request(req).await?,
    x.ok_or(Error::from("Unable to initiate connection"))?.clone(),
  )
  .await
  .map_err(|e| Error::from(format!("{}", e)))?;
  let resp = make_request(resp)
    .await
    .map_err(|x| Error::from(format!("Some error occurred while making req: {}", x)))?;
  Ok(resp)
}

async fn get_option() -> Option<Arc<ServerContext>> {
  SERV_CTX.read().await.clone()
}

async fn make_request(response: hyper::Response<hyper::Body>) -> anyhow::Result<Response> {
  let buf = hyper::body::to_bytes(response).await?;
  let text = std::str::from_utf8(&buf)?;
  let mut response =
    Response::ok(text).map_err(|x| anyhow!("Some error occurred while making req: {}", x.to_string()))?;
  response
    .headers_mut()
    .append("Content-Type", "text/html")
    .map_err(|x| anyhow!("Some error occurred while making req: {}", x.to_string()))?;
  Ok(response)
}
async fn make_req() -> anyhow::Result<Config> {
  let reader = ConfigReader::init(
    ["https://raw.githubusercontent.com/tailcallhq/tailcall/main/examples/jsonplaceholder.graphql"].iter(),
  );
  reader.read().await
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
  builder
    .body(hyper::body::Body::from(body))
    .map_err(|e| Error::from(format!("{}", e)))
}
