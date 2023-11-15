use std::collections::BTreeSet;
use std::sync::Arc;

use anyhow::Result;
use async_graphql::http::GraphiQLSource;
use client::DefaultHttpClient;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, HeaderMap, Request, Response, StatusCode};

use super::request_context::RequestContext;
use super::ServerContext;
use crate::async_graphql_hyper;
use crate::blueprint::Blueprint;
use crate::cli::CLIError;
use crate::config::Config;
use crate::http::client;

fn graphiql() -> Result<Response<Body>> {
    Ok(Response::new(Body::from(
        GraphiQLSource::build().endpoint("/graphql").finish(),
    )))
}

pub async fn graphql_request(req: Request<Body>, server_ctx: &ServerContext) -> Result<Response<Body>> {
    let upstream = server_ctx.blueprint.upstream.clone();
    let allowed = upstream.get_allowed_headers();
    let headers = create_allowed_headers(req.headers(), &allowed);
    let bytes = hyper::body::to_bytes(req.into_body()).await?;
    let request: async_graphql_hyper::GraphQLRequest = serde_json::from_slice(&bytes)?;
    let req_ctx = Arc::new(RequestContext::from(server_ctx).req_headers(headers));
    let mut response = request
        .data(req_ctx.clone())
        .execute(&server_ctx.schema)
        .await;
    if server_ctx.blueprint.server.enable_cache_control_header {
        if let Some(ttl) = req_ctx.get_min_max_age() {
            response = response.set_cache_control(ttl as i32);
        }
    }
    let mut resp = response.to_response()?;
    if !server_ctx.blueprint.server.response_headers.is_empty() {
        resp
            .headers_mut()
            .extend(server_ctx.blueprint.server.response_headers.clone());
    }

    Ok(resp)
}

fn not_found() -> Result<Response<Body>> {
    Ok(Response::builder().status(StatusCode::NOT_FOUND).body(Body::empty())?)
}

async fn handle_request(req: Request<Body>, state: Arc<ServerContext>) -> Result<Response<Body>> {
    match *req.method() {
        hyper::Method::GET if state.blueprint.server.enable_graphiql => graphiql(),
        hyper::Method::POST if req.uri().path() == "/graphql" => graphql_request(req, state.as_ref()).await,
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

pub async fn start_server_with_url(config: &Config) -> Result<()> {
    let blueprint = Blueprint::try_from(config).map_err(CLIError::from)?;
    let http_client = Arc::new(DefaultHttpClient::new(&blueprint.upstream));
    let state = Arc::new(ServerContext::new(
        blueprint.clone(),
        http_client,
    ));
    let make_svc = make_service_fn(move |_conn| {
        let state = Arc::clone(&state);
        async move { Ok::<_, anyhow::Error>(service_fn(move |req| handle_request(req, state.clone()))) }
    });
    let addr = (blueprint.server.hostname, blueprint.server.port).into();
    let server = hyper::Server::try_bind(&addr).map_err(CLIError::from)?.serve(make_svc);
    log::info!("🚀 Tailcall launched at [{}]", addr);
    if blueprint.server.enable_graphiql {
        log::info!("🌍 Playground: http://{}", addr);
    }

    Ok(server.await.map_err(CLIError::from)?)
}