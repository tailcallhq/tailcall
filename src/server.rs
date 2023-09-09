use std::collections::BTreeMap;
use std::fs;
use std::sync::Arc;

use anyhow::Result;
use async_graphql::dynamic;
use async_graphql::http::GraphiQLSource;
use hyper::header::CONTENT_TYPE;
use hyper::http::HeaderValue;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, StatusCode};

use crate::async_graphql_hyper;
use crate::blueprint::Blueprint;
use crate::cache_control::{min, set_cache_control};
use crate::cli::CLIError;
use crate::config::{Config, Server};
use crate::http::{HttpClient, HttpDataLoader};

fn graphiql() -> Result<Response<Body>> {
    Ok(Response::new(Body::from(
        GraphiQLSource::build().endpoint("/graphql").finish(),
    )))
}

#[derive(Clone)]
struct AppState {
    schema: Arc<dynamic::Schema>,
    client: Arc<HttpClient>,
    allowed_headers: Arc<AllowedHeaders>,
    enable_graphiql: Option<String>,
    server: Server,
}

async fn graphql_request(req: Request<Body>, state: &AppState) -> Result<Response<Body>> {
    let forwarded_headers = if state.allowed_headers.header_names.is_some() {
        let all_headers = AllHeaders::try_from(&req)?;
        AllowedHeaders::filter_allowed_headers(&all_headers, &state.allowed_headers)
    } else {
        BTreeMap::new()
    };

    let bytes = hyper::body::to_bytes(req.into_body()).await?;
    let request: async_graphql_hyper::GraphQLRequest = serde_json::from_slice(&bytes)?;

    let client = state.client.as_ref().to_owned();
    let loader = Arc::new(
        HttpDataLoader::new(client.clone())
            .headers(forwarded_headers)
            .to_async_data_loader_without_delay(),
    );

    let mut executed_response = request
        .data(loader.clone())
        .data(client.clone())
        .data(state.server.clone())
        .execute(state.schema.as_ref())
        .await;

    if client.enable_cache_control {
        let ttls: &Vec<Option<u64>> = &loader.get_cached_values().values().map(|x| x.ttl).collect();
        executed_response = set_cache_control(executed_response, min(ttls).unwrap_or(0) as i32);
        executed_response.into_hyper_response()
    } else {
        let body = serde_json::to_string(&executed_response)?;
        Ok(Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
            .body(Body::from(body))?)
    }
}

fn not_found() -> Result<Response<Body>> {
    Ok(Response::builder().status(StatusCode::NOT_FOUND).body(Body::empty())?)
}

async fn handle_request(req: Request<Body>, state: Arc<AppState>) -> Result<Response<Body>> {
    match *req.method() {
        hyper::Method::GET if state.enable_graphiql.as_ref() == Some(&req.uri().path().to_string()) => graphiql(),
        hyper::Method::POST if req.uri().path() == "/graphql" => graphql_request(req, state.as_ref()).await,
        _ => not_found(),
    }
}

pub async fn start_server(file_path: &String) -> Result<()> {
    let server_sdl = fs::read_to_string(file_path)?;
    let config = Config::from_sdl(&server_sdl)?;
    println!("{}", serde_json::to_string_pretty(&config.server).unwrap());
    let port = config.port();
    let enable_http_cache = config.server.enable_http_cache();
    let enable_cache_control = config.server.enable_cache_control();
    let proxy = config.proxy();
    let server = config.server.clone();
    let blueprint = Blueprint::try_from(&config).map_err(CLIError::from)?;

    let state = Arc::new(AppState {
        schema: Arc::new(blueprint.to_schema(&server)?),
        client: Arc::new(HttpClient::new(enable_http_cache, proxy, enable_cache_control)),
        allowed_headers: Arc::new(AllowedHeaders::new(&server.allowed_headers.clone())),
        enable_graphiql: config.server.enable_graphiql.clone(),
        server: config.server.clone(),
    });

    let make_svc = make_service_fn(move |_conn| {
        let state = Arc::clone(&state);
        async move { Ok::<_, anyhow::Error>(service_fn(move |req| handle_request(req, state.clone()))) }
    });

    let addr = ([0, 0, 0, 0], port).into();
    let server = hyper::Server::try_bind(&addr).map_err(CLIError::from)?.serve(make_svc);
    Ok(server.await.map_err(CLIError::from)?)
}

pub struct AllowedHeaders {
    pub header_names: Option<Vec<String>>,
}

impl AllowedHeaders {
    pub fn new(allowed_headers: &Option<Vec<String>>) -> Self {
        Self { header_names: allowed_headers.clone() }
    }

    pub fn filter_allowed_headers(
        all_headers: &AllHeaders,
        allowed_headers: &Arc<AllowedHeaders>,
    ) -> BTreeMap<String, String> {
        let mut forwarded_headers = BTreeMap::new();
        if let Some(allowed_names) = &allowed_headers.header_names {
            for (k, v) in all_headers.header_map.iter() {
                if allowed_names.contains(k) {
                    forwarded_headers.insert(k.clone(), v.clone());
                }
            }
        }
        forwarded_headers
    }
}

pub struct AllHeaders {
    pub header_map: BTreeMap<String, String>,
}

impl TryFrom<&Request<Body>> for AllHeaders {
    type Error = anyhow::Error;

    fn try_from(value: &Request<Body>) -> std::result::Result<Self, Self::Error> {
        let mut header_map = BTreeMap::new();
        for (k, v) in value.headers().iter() {
            if let Ok(value) = v.to_str() {
                header_map.insert(k.as_str().to_string(), value.to_string());
            }
        }
        Ok(Self { header_map })
    }
}
