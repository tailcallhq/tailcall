use std::collections::BTreeSet;
use std::sync::Arc;

use anyhow::Result;
use async_graphql::http::GraphiQLSource;
use hyper::{Body, HeaderMap, Request, Response, StatusCode};

use super::request_context::RequestContext;
use super::ServerContext;
use crate::async_graphql_hyper::{GraphQLRequest, GraphQLResponse};
use crate::http::request_handlers::api_request_handler::api_request;
use crate::http::request_handlers::graphql_request_handler::{graphql_batch_request, graphql_single_request};

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

pub fn create_request_context(req: &Request<Body>, server_ctx: &ServerContext) -> RequestContext {
  let upstream = server_ctx.blueprint.upstream.clone();
  let allowed = upstream.get_allowed_headers();
  let headers = create_allowed_headers(req.headers(), &allowed);
  RequestContext::from(server_ctx).req_headers(headers)
}

pub fn update_cache_control_header(
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

fn create_allowed_headers(headers: &HeaderMap, allowed: &BTreeSet<String>) -> HeaderMap {
  let mut new_headers = HeaderMap::new();
  for (k, v) in headers.iter() {
    if allowed.contains(k.as_str()) {
      new_headers.insert(k, v.clone());
    }
  }

  new_headers
}

pub async fn handle_single_request(req: Request<Body>, state: Arc<ServerContext>) -> Result<Response<Body>> {
  match *req.method() {
    hyper::Method::POST if req.uri().path() == "/graphql" => graphql_single_request(req, state.as_ref()).await,
    hyper::Method::GET if req.uri().path().starts_with("/api") => {
      api_request::<GraphQLRequest>(req, state.as_ref()).await
    }
    hyper::Method::GET if state.blueprint.server.enable_graphiql => graphiql(),
    _ => not_found(),
  }
}

pub async fn handle_batch_request(req: Request<Body>, state: Arc<ServerContext>) -> Result<Response<Body>> {
  match *req.method() {
    hyper::Method::POST if req.uri().path() == "/graphql" => graphql_batch_request(req, state.as_ref()).await,
    hyper::Method::GET if req.uri().path().starts_with("/api") => {
      api_request::<GraphQLRequest>(req, state.as_ref()).await
    }
    hyper::Method::GET if state.blueprint.server.enable_graphiql => graphiql(),
    _ => not_found(),
  }
}
