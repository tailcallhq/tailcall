use std::ops::Deref;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use anyhow::anyhow;
use lazy_static::lazy_static;
use tailcall::async_graphql_hyper::GraphQLRequest;
use tailcall::blueprint::Blueprint;
use tailcall::config::reader::ConfigReader;
use tailcall::config::Config;
use tailcall::http::{handle_request, AppContext};
use tailcall::io::EnvIO;

use crate::{init_env, init_file, init_http};

lazy_static! {
  static ref APP_CTX: RwLock<Option<Arc<AppContext>>> = RwLock::new(None);
}

///
/// Reads the configuration from the CONFIG environment variable.
///
async fn get_config(env_io: &impl EnvIO, env: Rc<worker::Env>) -> anyhow::Result<Config> {
  let path = env_io.get("CONFIG").ok_or(anyhow!("CONFIG var is not set"))?;
  let file_io = init_file(env.clone());
  let http_io = init_http();
  let reader = ConfigReader::init(file_io, http_io);
  let config = reader.read(&[path]).await?;
  Ok(config)
}

pub async fn fetch(req: worker::Request, env: worker::Env, _: worker::Context) -> anyhow::Result<worker::Response> {
  let env = Rc::new(env);
  log::debug!("Execution starting");
  let app_ctx = init(env).await?;
  let resp = handle_request::<GraphQLRequest>(to_request(req).await?, app_ctx).await?;
  Ok(to_response(resp).await?)
}

///
/// Initializes the worker once and caches the app context
/// for future requests.
///
async fn init(env: Rc<worker::Env>) -> anyhow::Result<Arc<AppContext>> {
  // Read context from cache
  if let Some(app_ctx) = read_app_ctx() {
    Ok(app_ctx)
  } else {
    // Create new context
    let env_io = init_env(env.clone());
    let cfg = get_config(&env_io, env.clone()).await?;
    let blueprint = Blueprint::try_from(&cfg)?;
    let h_client = Arc::new(init_http());

    let app_ctx = Arc::new(AppContext::new(blueprint, h_client.clone(), h_client, Arc::new(env_io)));
    *APP_CTX.write().unwrap() = Some(app_ctx.clone());
    log::info!("Initialized new application context");
    Ok(app_ctx)
  }
}

fn read_app_ctx() -> Option<Arc<AppContext>> {
  APP_CTX.read().unwrap().clone()
}

pub async fn to_response(response: hyper::Response<hyper::Body>) -> anyhow::Result<worker::Response> {
  let status = response.status().as_u16();
  let headers = response.headers().clone();
  let bytes = hyper::body::to_bytes(response).await?;
  let body = worker::ResponseBody::Body(bytes.to_vec());
  let mut w_response = worker::Response::from_body(body).map_err(to_anyhow)?;
  w_response = w_response.with_status(status);
  let mut_headers = w_response.headers_mut();
  for (name, value) in headers.iter() {
    let value = String::from_utf8(value.as_bytes().to_vec())?;
    mut_headers.append(name.as_str(), &value).map_err(to_anyhow)?;
  }

  Ok(w_response)
}

fn to_method(method: worker::Method) -> anyhow::Result<hyper::Method> {
  let method = &*method.to_string().to_uppercase();
  match method {
    "GET" => Ok(hyper::Method::GET),
    "POST" => Ok(hyper::Method::POST),
    "PUT" => Ok(hyper::Method::PUT),
    "DELETE" => Ok(hyper::Method::DELETE),
    "HEAD" => Ok(hyper::Method::HEAD),
    "OPTIONS" => Ok(hyper::Method::OPTIONS),
    "PATCH" => Ok(hyper::Method::PATCH),
    "CONNECT" => Ok(hyper::Method::CONNECT),
    "TRACE" => Ok(hyper::Method::TRACE),
    method => Err(anyhow!("Unsupported HTTP method: {}", method)),
  }
}

pub async fn to_request(mut req: worker::Request) -> anyhow::Result<hyper::Request<hyper::Body>> {
  let body = req.text().await.map_err(to_anyhow)?;
  let method = req.method();
  let uri = req.url().map_err(to_anyhow)?.as_str().to_string();
  let headers = req.headers();
  let mut builder = hyper::Request::builder().method(to_method(method)?).uri(uri);
  for (k, v) in headers {
    builder = builder.header(k, v);
  }
  Ok(builder.body(hyper::body::Body::from(body))?)
}

fn to_anyhow<T: std::fmt::Display>(e: T) -> anyhow::Error {
  anyhow!("{}", e)
}
