use std::borrow::Cow;
use std::collections::BTreeSet;
use std::sync::Arc;

use anyhow::Result;
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::ServerError;
use hyper::{Body, header, HeaderMap, Request, Response, StatusCode};
use hyper::header::HeaderValue;
use hyper::http::{HeaderName, Method};
use hyper::http::request::Parts;
use serde::de::DeserializeOwned;
use tower_http::cors::{AllowCredentials, AllowHeaders, AllowMethods, AllowOrigin, AllowPrivateNetwork, CorsLayer, ExposeHeaders, MaxAge, Vary};

use super::request_context::RequestContext;
use super::{showcase, AppContext};
use crate::async_graphql_hyper::{GraphQLRequestLike, GraphQLResponse};
use crate::blueprint::CorsParams;

pub fn graphiql(req: &Request<Body>) -> Result<Response<Body>> {
    let query = req.uri().query();
    let endpoint = "/graphql";
    let endpoint = if let Some(query) = query {
        if query.is_empty() {
            Cow::Borrowed(endpoint)
        } else {
            Cow::Owned(format!("{}?{}", endpoint, query))
        }
    } else {
        Cow::Borrowed(endpoint)
    };

    Ok(Response::new(Body::from(playground_source(
        GraphQLPlaygroundConfig::new(&endpoint).title("Tailcall - GraphQL IDE"),
    ))))
}

fn not_found() -> Result<Response<Body>> {
    Ok(Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::empty())?)
}

fn create_request_context(req: &Request<Body>, app_ctx: &AppContext) -> RequestContext {
    let upstream = app_ctx.blueprint.upstream.clone();
    let allowed = upstream.allowed_headers;
    let headers = create_allowed_headers(req.headers(), &allowed);
    RequestContext::from(app_ctx).req_headers(headers)
}

fn update_cache_control_header(
    response: GraphQLResponse,
    app_ctx: &AppContext,
    req_ctx: Arc<RequestContext>,
) -> GraphQLResponse {
    if app_ctx.blueprint.server.enable_cache_control_header {
        let ttl = req_ctx.get_min_max_age().unwrap_or(0);
        let cache_public_flag = req_ctx.is_cache_public().unwrap_or(true);
        return response.set_cache_control(ttl, cache_public_flag);
    }
    response
}

pub fn update_response_headers(resp: &mut hyper::Response<hyper::Body>, app_ctx: &AppContext) {
    if !app_ctx.blueprint.server.response_headers.is_empty() {
        resp.headers_mut()
            .extend(app_ctx.blueprint.server.response_headers.clone());
    }
}

pub async fn graphql_request<T: DeserializeOwned + GraphQLRequestLike>(
    req: Request<Body>,
    app_ctx: &AppContext,
) -> Result<Response<Body>> {
    let req_ctx = Arc::new(create_request_context(&req, app_ctx));
    let bytes = hyper::body::to_bytes(req.into_body()).await?;
    let request = serde_json::from_slice::<T>(&bytes);
    match request {
        Ok(request) => {
            let mut response = request.data(req_ctx.clone()).execute(&app_ctx.schema).await;
            response = update_cache_control_header(response, app_ctx, req_ctx);
            let mut resp = response.to_response()?;
            update_response_headers(&mut resp, app_ctx);
            Ok(resp)
        }
        Err(err) => {
            log::error!(
                "Failed to parse request: {}",
                String::from_utf8(bytes.to_vec()).unwrap()
            );

            let mut response = async_graphql::Response::default();
            let server_error =
                ServerError::new(format!("Unexpected GraphQL Request: {}", err), None);
            response.errors = vec![server_error];

            Ok(GraphQLResponse::from(response).to_response()?)
        }
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


pub async fn handle_request_with_cors<T: DeserializeOwned + GraphQLRequestLike>(
    req: Request<Body>,
    layer: &CorsParams,
    app_ctx: Arc<AppContext>
) -> Result<Response<Body>> {
    let (parts, body) = req.into_parts();
    let origin = parts.headers.get(&header::ORIGIN);

    let mut headers = HeaderMap::new();

    // These headers are applied to both preflight and subsequent regular CORS requests:
    // https://fetch.spec.whatwg.org/#http-responses

    headers.extend(layer.allow_origin_to_header(origin));
    headers.extend(layer.allow_credentials_to_header());
    headers.extend(layer.allow_private_network_to_header(&parts));

    let mut vary_headers = layer.vary.iter().cloned();
    if let Some(first) = vary_headers.next() {
        let mut header = match headers.entry(header::VARY) {
            header::Entry::Occupied(_) => {
                unreachable!("no vary header inserted up to this point")
            }
            header::Entry::Vacant(v) => v.insert_entry(first),
        };

        for val in vary_headers {
            header.append(val);
        }
    }

    // Return results immediately upon preflight request
    if parts.method == Method::OPTIONS {
        // These headers are applied only to preflight requests
        headers.extend(layer.allow_methods_to_header(&parts));
        headers.extend(layer.allow_headers_to_header(&parts));
        headers.extend(layer.max_age_to_header());

        let mut response = Response::new(Body::default());
        std::mem::swap(response.headers_mut(), &mut headers);

        return Ok(response)
    } else {
        // This header is applied only to non-preflight requests
        headers.extend(layer.expose_headers_to_header());

        let req = Request::from_parts(parts, body);
        let mut response = handle_request::<T>(req, app_ctx).await?;

        let response_headers = response.headers_mut();

        // vary header can have multiple values, don't overwrite
        // previously-set value(s).
        if let Some(vary) = headers.remove(header::VARY) {
            response_headers.append(header::VARY, vary);
        }
        // extend will overwrite previous headers of remaining names
        response_headers.extend(headers.drain());

        Ok(response)
    }
}

pub async fn handle_request<T: DeserializeOwned + GraphQLRequestLike>(
    req: Request<Body>,
    app_ctx: Arc<AppContext>,
) -> Result<Response<Body>> {
    match *req.method() {
        // NOTE:
        // The first check for the route should be for `/graphql`
        // This is always going to be the most used route.
        hyper::Method::POST if req.uri().path() == "/graphql" => {
            graphql_request::<T>(req, app_ctx.as_ref()).await
        }
        hyper::Method::POST
            if app_ctx.blueprint.server.enable_showcase
                && req.uri().path() == "/showcase/graphql" =>
        {
            let app_ctx =
                match showcase::create_app_ctx::<T>(&req, app_ctx.runtime.clone(), false).await? {
                    Ok(app_ctx) => app_ctx,
                    Err(res) => return Ok(res),
                };

            graphql_request::<T>(req, &app_ctx).await
        }

        hyper::Method::GET if app_ctx.blueprint.server.enable_graphiql => graphiql(&req),
        _ => not_found(),
    }
}
