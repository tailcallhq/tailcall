use std::borrow::Cow;
use std::collections::BTreeSet;
use std::sync::Arc;

use anyhow::Result;
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::ServerError;
use hyper::header::CONTENT_TYPE;
use hyper::{Body, HeaderMap, Request, Response, StatusCode};
use prometheus::{Encoder, ProtobufEncoder, TextEncoder, PROTOBUF_FORMAT, TEXT_FORMAT};
use serde::de::DeserializeOwned;
use tracing::instrument;

use super::request_context::RequestContext;
use super::{showcase, AppContext};
use crate::async_graphql_hyper::{GraphQLRequestLike, GraphQLResponse};
use crate::blueprint::telemetry::TelemetryExporter;
use crate::config::{PrometheusExporter, PrometheusFormat};

const API_URL_PREFIX: &str = "/api";
const ENDPOINT: &str = "/graphql";
const SHOWCASE_ENDPOINT: &str = "/showcase/graphql";

fn get_graphql_playground_endpoint(req: &Request<Body>) -> Cow<'static, str> {
    req.uri().query().map_or_else(
        || Cow::Borrowed(ENDPOINT),
        |query| {
            if query.is_empty() {
                Cow::Borrowed(ENDPOINT)
            } else {
                Cow::Owned(format!("{}?{}", ENDPOINT, query))
            }
        },
    )
}

pub fn graphiql(req: &Request<Body>) -> Result<Response<Body>> {
    let endpoint = get_graphql_playground_endpoint(req);
    Ok(Response::new(Body::from(playground_source(
        GraphQLPlaygroundConfig::new(&endpoint).title("Tailcall - GraphQL IDE"),
    ))))
}

fn prometheus_metrics(prometheus_exporter: &PrometheusExporter) -> Result<Response<Body>> {
    let metric_families = prometheus::default_registry().gather();
    let mut buffer = vec![];

    match prometheus_exporter.format {
        PrometheusFormat::Text => TextEncoder::new().encode(&metric_families, &mut buffer)?,
        PrometheusFormat::Protobuf => {
            ProtobufEncoder::new().encode(&metric_families, &mut buffer)?
        }
    };

    let content_type = match prometheus_exporter.format {
        PrometheusFormat::Text => TEXT_FORMAT,
        PrometheusFormat::Protobuf => PROTOBUF_FORMAT,
    };

    Ok(Response::builder()
        .status(200)
        .header(CONTENT_TYPE, content_type)
        .body(Body::from(buffer))?)
}

fn not_found() -> Result<Response<Body>> {
    Ok(Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::empty())?)
}

fn create_request_context(req: &Request<Body>, app_ctx: &AppContext) -> RequestContext {
    let upstream = app_ctx.blueprint.upstream.clone();
    let allowed = upstream.allowed_headers;
    let req_headers = create_allowed_headers(req.headers(), &allowed);

    let allowed = app_ctx.blueprint.server.get_experimental_headers();
    let experimental_headers = create_allowed_headers(req.headers(), &allowed);
    RequestContext::from(app_ctx)
        .req_headers(req_headers)
        .experimental_headers(experimental_headers)
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

pub fn update_response_headers(
    resp: &mut Response<Body>,
    cookie_headers: Option<HeaderMap>,
    app_ctx: &AppContext,
    req_ctx: Arc<RequestContext>,
) {
    if let Some(cookie_headers) = cookie_headers {
        resp.headers_mut().extend(cookie_headers);
    }
    if !app_ctx.blueprint.server.experimental_headers.is_empty() {
        resp.headers_mut()
            .extend(req_ctx.experimental_headers.clone());
    }
    resp.headers_mut()
        .extend(app_ctx.blueprint.server.response_headers.clone());
}

fn create_allowed_headers(headers: &HeaderMap, allowed: &BTreeSet<String>) -> HeaderMap {
    let mut new_headers = HeaderMap::new();
    for (k, v) in headers.iter() {
        if allowed
            .iter()
            .any(|allowed_key| allowed_key.eq_ignore_ascii_case(k.as_str()))
        {
            new_headers.insert(k, v.clone());
        }
    }
    new_headers
}

async fn get_response<T: DeserializeOwned + GraphQLRequestLike>(
    request: T,
    app_ctx: &AppContext,
    request_context: Arc<RequestContext>,
) -> Result<Response<Body>> {
    let mut response = request
        .data(request_context.clone())
        .execute(&app_ctx.schema)
        .await;

    let cookie_headers = request_context.cookie_headers.clone();

    response = update_cache_control_header(response, app_ctx, request_context.clone());
    let mut resp = response.to_response()?;

    update_response_headers(
        &mut resp,
        cookie_headers.map(|v| v.lock().unwrap().clone()),
        app_ctx,
        request_context,
    );

    Ok(resp)
}

pub async fn graphql_request<T: DeserializeOwned + GraphQLRequestLike>(
    req: Request<Body>,
    app_ctx: &AppContext,
) -> Result<Response<Body>> {
    let req_ctx = Arc::new(create_request_context(&req, app_ctx));
    let bytes = hyper::body::to_bytes(req.into_body()).await?;
    let graphql_request = serde_json::from_slice::<T>(&bytes);
    match graphql_request {
        Ok(request) => {
            let resp = get_response(request, app_ctx, req_ctx).await?;
            Ok(resp)
        }
        Err(err) => {
            tracing::error!(
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

async fn handle_rest_apis(
    mut request: Request<Body>,
    app_ctx: Arc<AppContext>,
) -> Result<Response<Body>> {
    let req_ctx = Arc::new(create_request_context(&request, app_ctx.as_ref()));
    *request.uri_mut() = request.uri().path().replace(API_URL_PREFIX, "").parse()?;
    if let Some(p_request) = app_ctx.endpoints.matches(&request) {
        let graphql_request = p_request.into_request(request).await?;
        let resp = get_response(graphql_request, app_ctx.as_ref(), req_ctx).await?;
        return Ok(resp);
    }

    not_found()
}

#[instrument(skip_all, err, fields(method = % req.method(), url = % req.uri()))]
pub async fn handle_request<T: DeserializeOwned + GraphQLRequestLike>(
    req: Request<Body>,
    mut app_ctx: Arc<AppContext>,
) -> Result<Response<Body>> {
    if req.uri().path().starts_with(API_URL_PREFIX) {
        return handle_rest_apis(req, app_ctx).await;
    }

    match *req.method() {
        // NOTE:
        // The first check for the route should be for `/graphql`
        // This is always going to be the most used route.
        hyper::Method::POST => match req.uri().path() {
            ENDPOINT | SHOWCASE_ENDPOINT => {
                if app_ctx.blueprint.server.enable_showcase {
                    app_ctx =
                        match showcase::create_app_ctx::<T>(&req, app_ctx.runtime.clone(), false)
                            .await?
                        {
                            Ok(app_ctx) => Arc::new(app_ctx),
                            Err(res) => return Ok(res),
                        };
                }
                graphql_request::<T>(req, app_ctx.as_ref()).await
            }
            _ => not_found(),
        },
        hyper::Method::GET => {
            if let Some(TelemetryExporter::Prometheus(prometheus)) =
                app_ctx.blueprint.opentelemetry.export.as_ref()
            {
                if req.uri().path() == prometheus.path {
                    return prometheus_metrics(prometheus);
                }
            };

            if app_ctx.blueprint.server.enable_graphiql {
                return graphiql(&req);
            }

            not_found()
        }
        _ => not_found(),
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_create_allowed_headers() {
        use std::collections::BTreeSet;

        use hyper::header::{HeaderMap, HeaderValue};

        use super::create_allowed_headers;

        let mut headers = HeaderMap::new();
        headers.insert("X-foo", HeaderValue::from_static("bar"));
        headers.insert("x-bar", HeaderValue::from_static("foo"));
        headers.insert("x-baz", HeaderValue::from_static("baz"));

        let allowed = BTreeSet::from_iter(vec!["x-foo".to_string(), "X-bar".to_string()]);

        let new_headers = create_allowed_headers(&headers, &allowed);
        assert_eq!(new_headers.len(), 2);
        assert_eq!(new_headers.get("x-foo").unwrap(), "bar");
        assert_eq!(new_headers.get("x-bar").unwrap(), "foo");
    }
}
