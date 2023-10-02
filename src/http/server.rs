use std::collections::HashSet;
use std::fs;
use std::sync::Arc;

use anyhow::Result;
use async_graphql::http::GraphiQLSource;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, HeaderMap, Request, Response, StatusCode};

use super::request_context::RequestContext;
use super::ServerContext;
use crate::async_graphql_hyper;
use crate::blueprint::Blueprint;
use crate::cli::CLIError;
use crate::config::{Config, Server};

fn graphiql() -> Result<Response<Body>> {
  Ok(Response::new(Body::from(
    GraphiQLSource::build().endpoint("/graphql").finish(),
  )))
}

async fn graphql_request(req: Request<Body>, server_ctx: &ServerContext) -> Result<Response<Body>> {
  let server = server_ctx.server.clone();
  let allowed = server.allowed_headers.unwrap_or_default();
  let headers = create_allowed_headers(req.headers(), &allowed);
  let bytes = hyper::body::to_bytes(req.into_body()).await?;
  let request: async_graphql_hyper::GraphQLRequest = serde_json::from_slice(&bytes)?;
  let req_ctx = Arc::new(RequestContext::from(server_ctx).req_headers(headers));
  let mut response = request.data(req_ctx.clone()).execute(&server_ctx.schema).await;

  if server_ctx.server.enable_cache_control() {
    let ttl = crate::http::min_ttl(req_ctx.get_cached_values().values());
    response = response.set_cache_control(ttl);
  }

  response = response.set_response_headers(server_ctx.server.response_headers.clone());
  response.to_response()
}
fn not_found() -> Result<Response<Body>> {
  Ok(Response::builder().status(StatusCode::NOT_FOUND).body(Body::empty())?)
}
async fn handle_request(req: Request<Body>, state: Arc<ServerContext>) -> Result<Response<Body>> {
  match *req.method() {
    hyper::Method::GET if state.server.enable_graphiql.as_ref() == Some(&req.uri().path().to_string()) => graphiql(),
    hyper::Method::POST if req.uri().path() == "/graphql" => graphql_request(req, state.as_ref()).await,
    _ => not_found(),
  }
}
fn create_allowed_headers(headers: &HeaderMap, allowed: &HashSet<String>) -> HeaderMap {
  let mut new_headers = HeaderMap::new();
  for (k, v) in headers.iter() {
    if allowed.contains(k.as_str()) {
      new_headers.insert(k, v.clone());
    }
  }

  new_headers
}

fn set_response_headers(server: &mut Server){
  server.set_response_headers();
}

pub async fn start_server(file_path: &String) -> Result<()> {
  let server_sdl = fs::read_to_string(file_path)?;
  let config = Config::from_sdl(&server_sdl)?;
  let port = config.port();
  let mut server = config.server.clone();
    set_response_headers(&mut server);
  let blueprint = Blueprint::try_from(&config).map_err(CLIError::from)?;
  let state = Arc::new(ServerContext::new(blueprint, server));
  let make_svc = make_service_fn(move |_conn| {
    let state = Arc::clone(&state);
    async move { Ok::<_, anyhow::Error>(service_fn(move |req| handle_request(req, state.clone()))) }
  });

  let addr = ([0, 0, 0, 0], port).into();
  let server = hyper::Server::try_bind(&addr).map_err(CLIError::from)?.serve(make_svc);

  Ok(server.await.map_err(CLIError::from)?)
}
