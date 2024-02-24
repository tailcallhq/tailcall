use std::borrow::Cow;
use std::collections::BTreeSet;
use std::sync::Arc;

use anyhow::{Context, Result};
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::ServerError;
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::{Request, Response, StatusCode};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::de::DeserializeOwned;

use super::request_context::RequestContext;
use super::{showcase, AppContext};
use crate::async_graphql_hyper::{GraphQLRequestLike, GraphQLResponse};

pub fn graphiql(req: &Request<Full<Bytes>>) -> Result<Response<Full<Bytes>>> {
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

    Ok(Response::new(Full::new(Bytes::from(playground_source(
        GraphQLPlaygroundConfig::new(&endpoint).title("Tailcall - GraphQL IDE"),
    )))))
}

fn not_found() -> Result<Response<Full<Bytes>>> {
    Ok(Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Full::new(Bytes::default()))?)
}

fn create_request_context(req: &Request<Full<Bytes>>, app_ctx: &AppContext) -> RequestContext {
    let upstream = app_ctx.blueprint.upstream.clone();
    let allowed = upstream.allowed_headers;
    let headers = create_allowed_headers(&to_reqwest_hmap(req.headers()), &allowed);
    RequestContext::from(app_ctx).req_headers(headers)
}

fn to_reqwest_hmap(hyper_headers: &hyper::HeaderMap) -> HeaderMap {
    let mut reqwest_headers = HeaderMap::new();
    for (key, value) in hyper_headers.iter() {
        if let (Ok(name), Ok(value_str)) = (
            HeaderName::from_bytes(key.as_str().as_bytes()),
            HeaderValue::from_str(value.to_str().unwrap_or_default()),
        ) {
            reqwest_headers.insert(name, value_str);
        }
    }
    reqwest_headers
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

pub fn update_response_headers(resp: &mut hyper::Response<Full<Bytes>>, app_ctx: &AppContext) {
    if !app_ctx.blueprint.server.response_headers.is_empty() {
        resp.headers_mut()
            .extend(app_ctx.blueprint.server.response_headers.clone());
    }
}

pub async fn graphql_request<T: DeserializeOwned + GraphQLRequestLike>(
    req: Request<Full<Bytes>>,
    app_ctx: &AppContext,
) -> Result<Response<Full<Bytes>>> {
    let req_ctx = Arc::new(create_request_context(&req, app_ctx));
    let bytes = req
        .into_body()
        .frame()
        .await
        .context("unable to extract frame")??
        .into_data()
        .map_err(|e| anyhow::anyhow!("{:?}", e))?;

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

pub async fn handle_request<T: DeserializeOwned + GraphQLRequestLike>(
    req: Request<Full<Bytes>>,
    app_ctx: Arc<AppContext>,
) -> Result<Response<Full<Bytes>>> {
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

#[cfg(test)]
mod test_req_handler {
    use anyhow::Context;
    use http_body_util::{BodyExt, Full};
    use crate::http::graphiql;
    use crate::http::request_handler::not_found;

    #[tokio::test]
    async fn test_graphiql() -> anyhow::Result<()> {
        let mut req = hyper::Request::new(Full::new(hyper::body::Bytes::new()));
        *req.uri_mut() = "http://localhost:19194/?config=examples/foo.graphql".parse().unwrap();
        let resp = graphiql(&req)?;

        let bytes = resp
            .into_body()
            .frame()
            .await
            .context("unable to extract frame")??
            .into_data()
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;

        let string = String::from_utf8(bytes.to_vec())?;

        assert!(string.contains("\"endpoint\":\"/graphql?config=examples/foo.graphql\""));
        assert!(string.contains("Tailcall - GraphQL IDE"));

        Ok(())
    }
    #[test]
    fn test_not_found() -> anyhow::Result<()> {
        let not_found = not_found()?;
        assert_eq!(404u16, not_found.status().as_u16());
        assert_eq!(hyper::http::version::Version::HTTP_11, not_found.version());
        Ok(())
    }
}