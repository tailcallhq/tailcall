use std::collections::HashSet;
use std::fs;
use std::sync::Arc;

use anyhow::Result;

use async_graphql::http::GraphiQLSource;
use hyper::header::CONTENT_TYPE;
use hyper::http::HeaderValue;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, HeaderMap, Request, Response, StatusCode};

use super::request_context::RequestContext;
use super::ServerContext;
use crate::async_graphql_hyper;
use crate::blueprint::Blueprint;
use crate::cache_control::{min, set_cache_control};
use crate::cli::CLIError;
use crate::config::Config;

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
    let req_ctx = Arc::new(RequestContext::new(server_ctx.client.clone(), &headers));
    let mut response = request.data(req_ctx.clone()).execute(&server_ctx.schema).await;
    if server_ctx.server.enable_cache_control() {
        let ttls: &Vec<Option<u64>> = &req_ctx.get_cached_values().values().map(|x| x.stats.min_ttl).collect();
        response = set_cache_control(response, min(ttls).unwrap_or(0) as i32);
        response.into_hyper_response()
    } else {
        let body = serde_json::to_string(&response)?;
        Ok(Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
            .body(Body::from(body))?)
    }
}
fn not_found() -> Result<Response<Body>> {
    Ok(Response::builder().status(StatusCode::NOT_FOUND).body(Body::empty())?)
}
async fn handle_request(req: Request<Body>, state: Arc<ServerContext>) -> Result<Response<Body>> {
    match *req.method() {
        hyper::Method::GET if state.server.enable_graphiql.as_ref() == Some(&req.uri().path().to_string()) => {
            graphiql()
        }
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
pub async fn start_server(file_path: &String) -> Result<()> {
    let server_sdl = fs::read_to_string(file_path)?;
    let config = Config::from_sdl(&server_sdl)?;
    let port = config.port();
    let server = config.server.clone();
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
