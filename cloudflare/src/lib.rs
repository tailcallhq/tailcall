use std::sync::{Arc, RwLock};

use lazy_static::lazy_static;
use tailcall::async_graphql_hyper::GraphQLRequest;
use tailcall::blueprint::Blueprint;
use tailcall::config::reader::ConfigReader;
use tailcall::config::Config;
use tailcall::http::{handle_request, AppContext};
use tailcall::io::{EnvIO, FileIO, HttpIO};
use worker::*;

mod env;
mod file;
mod http;

fn init_env(env: Arc<Env>) -> impl EnvIO {
  env::EnvCloudflare::init(env)
}

fn init_file(r2_id: &str, env: Arc<Env>) -> Result<impl FileIO> {
  file::CloudflareFileIO::init(r2_id, env).map_err(conv_err)
}

fn init_http() -> impl HttpIO + Default + Clone {
  http::HttpCloudflare::init()
}

lazy_static! {
  static ref APP_CTX: RwLock<Option<Arc<AppContext>>> = RwLock::new(None);
}

async fn get_config(env_io: &impl EnvIO, env: Arc<Env>) -> Result<Config> {
  let config_val = env_io.get("CONFIG").ok_or(Error::from("Invalid path string"))?;
  let (r2_id, path) = separate_id_path(config_val).ok_or(Error::from("Invalid path string"))?;

  let file_io = init_file(&r2_id, env.clone())?;
  let http_io = init_http();
  let reader = ConfigReader::init(file_io, http_io);
  reader.read(&[path]).await.map_err(conv_err)
}

#[event(fetch)]
async fn main(req: Request, env: Env, _: Context) -> Result<Response> {
  let mut app_ctx = get_option().await;
  let env = Arc::new(env);
  if app_ctx.is_none() {
    app_ctx = Some(init(env).await?);
  }

  let resp = handle_request::<GraphQLRequest>(
    convert_to_hyper_request(req).await?,
    app_ctx.ok_or(Error::from("Unable to initiate connection"))?.clone(),
  )
  .await
  .map_err(conv_err)?;
  let resp = make_request(resp).await.map_err(conv_err)?;
  Ok(resp)
}

async fn init(env: Arc<Env>) -> Result<Arc<AppContext>> {
  let env_io = init_env(env.clone());
  let cfg = get_config(&env_io, env.clone()).await.map_err(conv_err)?;
  let blueprint = Blueprint::try_from(&cfg).map_err(conv_err)?;
  let universal_http_client = Arc::new(init_http());
  let http2_only_client = Arc::new(init_http());

  let app_ctx = Arc::new(AppContext::new(
    blueprint,
    universal_http_client,
    http2_only_client,
    Arc::new(env_io),
  ));
  *APP_CTX.write().unwrap() = Some(app_ctx.clone());
  Ok(app_ctx)
}

async fn get_option() -> Option<Arc<AppContext>> {
  APP_CTX.read().unwrap().clone()
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

fn separate_id_path(val: String) -> Option<(String, String)> {
  let mut split = val.split("/");
  let r2_id = split.next()?.to_string();
  let path = split.collect::<Vec<&str>>().join("/");
  Some((r2_id, path))
}
