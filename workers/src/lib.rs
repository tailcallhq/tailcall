use std::sync::{Arc, RwLock};

use lazy_static::lazy_static;
use tailcall::async_graphql_hyper::GraphQLRequest;
use tailcall::blueprint::Blueprint;
use tailcall::config::Config;
use tailcall::http::{handle_request, DefaultHttpClient, ServerContext};
use worker::Fetch::Url;
use worker::*;

/*use std::sync::Arc;
use hyper::Body;
use tailcall::async_graphql_hyper::GraphQLRequest;
use tailcall::blueprint::Blueprint;
use tailcall::cli::CLIError;
use tailcall::config::Config;
use tailcall::http::{DefaultHttpClient, handle_request, ServerContext};
*/
lazy_static! {
  static ref SERV_CTX: RwLock<Option<Arc<ServerContext>>> = RwLock::new(None);
}
// #[event(f)]

#[event(start)]
fn start() {
  /*    let u = "https://raw.githubusercontent.com/tailcallhq/tailcall/main/examples/jsonplaceholder.graphql";
      let window = web_sys::window().unwrap();
      async_std::task::block_on(async move {
          let mut resp = Url(
              "https://httpbin.org/anything"
                  .parse().unwrap(),
          )
              .send().await.unwrap();
          let txt = resp.text().await.unwrap();
          *SDL.write().unwrap() = format!("{:#?}", txt);
      });
  */

  // *SDL.write().unwrap() = format!("{:?}",x);
  /*
  async_std::task::block_on(async {
        // let client = reqwest::Client::new();
        // let response = client.get(url).send().await.unwrap();
        let body = reqwest::get("https://www.rust-lang.org").await;

    });*/
}

#[event(fetch)]
async fn main(req: Request, env: Env, ctx: Context) -> Result<Response> {
  let x = get_option();
  if x.is_none() {
    let cfg = Config::from_sdl(&make_req().await.unwrap()).to_result().unwrap();
    let blueprint = Blueprint::try_from(&cfg).unwrap();
    let http_client = Arc::new(DefaultHttpClient::new(&blueprint.upstream));
    let serv_ctx = Arc::new(ServerContext::new(blueprint, http_client));
    *SERV_CTX.write().unwrap() = Some(serv_ctx);
    return Response::ok(cfg.to_sdl());
  }
  let x = handle_request::<GraphQLRequest>(convert_to_hyper_request(req).await, x.unwrap().clone())
    .await
    .unwrap();
  let resp = make_request(x).await;
  Ok(resp)
}

fn get_option() -> Option<Arc<ServerContext>> {
  SERV_CTX.read().unwrap().clone()
}

async fn make_request(response: hyper::Response<hyper::Body>) -> Response {
  let buf = hyper::body::to_bytes(response).await.unwrap();
  let text = std::str::from_utf8(&buf).unwrap();
  let mut response = Response::ok(text).unwrap();
  response.headers_mut().append("Content-Type", "text/html").unwrap();
  response
}

async fn make_req() -> Result<String> {
  let mut resp = Url(
    "https://raw.githubusercontent.com/tailcallhq/tailcall/main/examples/jsonplaceholder.graphql"
      .parse()
      .unwrap(),
  )
  .send()
  .await?;
  let txt = resp.text().await?;
  Ok(txt)
}

fn convert_method(worker_method: Method) -> hyper::Method {
  let method_str = &*worker_method.to_string();

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

async fn convert_to_hyper_request(mut worker_request: Request) -> hyper::Request<hyper::Body> {
  let body = worker_request.text().await.unwrap();
  let method = worker_request.method();
  let uri = worker_request.url().unwrap().as_str().to_string();
  let headers = worker_request.headers();
  let mut builder = hyper::Request::builder().method(convert_method(method)).uri(uri);
  for (k, v) in headers {
    builder = builder.header(k, v);
  }
  builder.body(hyper::body::Body::from(body)).unwrap()
}
