use std::borrow::Cow;
use std::collections::BTreeSet;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::ServerError;
use async_graphql_value::{ConstValue, Variables};
use hyper::{Body, HeaderMap, Request, Response, StatusCode};
use routerify::prelude::RequestExt;
use routerify::{RequestService, RequestServiceBuilder, Router};
use serde::de::DeserializeOwned;

use super::request_context::RequestContext;
use super::{showcase, AppContext, Method};
use crate::async_graphql_hyper::{GraphQLRequest, GraphQLRequestLike, GraphQLResponse};

pub fn graphiql(req: Request<Body>) -> Result<Response<Body>> {
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
    app_ctx: Arc<AppContext>,
) -> Result<Response<Body>> {
    let req_ctx = Arc::new(create_request_context(&req, app_ctx.as_ref()));
    let bytes = hyper::body::to_bytes(req.into_body()).await?;
    let request = serde_json::from_slice::<T>(&bytes);
    match request {
        Ok(request) => {
            let mut response = request.data(req_ctx.clone()).execute(&app_ctx.schema).await;
            response = update_cache_control_header(response, app_ctx.as_ref(), req_ctx);
            let mut resp = response.to_response()?;
            update_response_headers(&mut resp, app_ctx.as_ref());
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

pub async fn graphql_query(
    req: Request<Body>,
    query: String,
    var_keys: Vec<String>,
    app_ctx: Arc<AppContext>,
) -> Result<Response<Body>> {
    let var_json = ConstValue::Object(
        var_keys
            .into_iter()
            .map(|key| {
                let val = req
                    .param(&key)
                    .ok_or(anyhow::anyhow!("`key` not provided"))?;
                Ok((
                    async_graphql::Name::new(key),
                    ConstValue::String(val.clone()),
                ))
            })
            .collect::<anyhow::Result<_>>()?,
    );

    println!("{var_json:?}");

    let request = async_graphql::Request::new(query).variables(Variables::from_value(var_json));
    let request = GraphQLRequest(request);
    let req_ctx = Arc::new(create_request_context(&req, app_ctx.as_ref()));
    let read_from_cache = Arc::new(Mutex::new(false));
    let response = request.data(req_ctx).execute(&app_ctx.schema).await;
    let mut resp = response.to_response()?;
    update_response_headers(&mut resp, app_ctx.as_ref());

    if *read_from_cache.lock().unwrap() {
        *resp.status_mut() = StatusCode::NOT_MODIFIED;
    }

    Ok(resp)
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

// pub async fn handle_request<B, E, T: DeserializeOwned + GraphQLRequestLike>(
//     req: Request<Body>,
//     router: Router<B, E>,
// ) -> Result<Response<Body>> {
//     router.
// }

pub fn create_request_service<T: DeserializeOwned + GraphQLRequestLike + Send + 'static>(
    app_ctx: Arc<AppContext>,
    remote_addr: SocketAddr,
) -> core::result::Result<RequestService<Body, anyhow::Error>, anyhow::Error> {
    let app_ctx_clone = app_ctx.clone();
    let mut builder = Router::builder().post("/graphql", move |req| {
        graphql_request::<T>(req, app_ctx_clone.clone())
    });

    let app_ctx_clone = app_ctx.clone();
    builder = builder.post("/showcase/graphql", move |req| {
        let app_ctx = app_ctx_clone.clone();
        async move {
            let app_ctx =
                match showcase::create_app_ctx::<T>(&req, app_ctx.clone().runtime.clone(), false)
                    .await?
                {
                    Ok(app_ctx) => app_ctx,
                    Err(res) => return Ok(res),
                };

            graphql_request::<T>(req, Arc::new(app_ctx)).await
        }
    });

    let app_ctx_clone = app_ctx.clone();
    builder = builder.get("/", move |req| {
        let app_ctx = app_ctx_clone.clone();
        async move {
            if app_ctx.blueprint.server.enable_graphiql {
                graphiql(req)
            } else {
                not_found()
            }
        }
    });

    for (rest, query) in app_ctx.blueprint.rest_apis.iter() {
        let app_ctx_clone = app_ctx.clone();
        let path = format!(
            "/api/{}",
            rest.path
                .strip_prefix('/')
                .unwrap_or(&rest.path)
                .replace('$', ":")
        );
        let var_keys: Vec<String> = rest.variables().map(|var| var.to_string()).collect();
        let query = query.clone();
        let handler =
            move |req| graphql_query(req, query.clone(), var_keys.clone(), app_ctx_clone.clone());

        builder = match rest.method {
            Method::GET => builder.get(path, handler),
            Method::POST => builder.post(path, handler),
            _ => builder,
        }
    }

    let router = builder.build().map_err(|e| anyhow::anyhow!("{e}"))?;

    let rs_builder = RequestServiceBuilder::new(router).map_err(|e| anyhow::anyhow!("{e}"))?;
    let request_service = rs_builder.build(remote_addr);
    Ok(request_service)
}
