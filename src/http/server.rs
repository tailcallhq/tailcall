use std::collections::{HashMap, HashSet};
use std::fs;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use anyhow::Result;
use async_graphql::http::GraphiQLSource;
use hyper::header::{HeaderValue, IF_NONE_MATCH};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, HeaderMap, Request, Response, StatusCode};
use tokio::time;
extern crate tokio;
use super::request_context::RequestContext;
use super::ServerContext;
use crate::async_graphql_hyper;
use crate::blueprint::Blueprint;
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
  let req_ctx = Arc::new(RequestContext::from(server_ctx).req_headers(headers));
  let mut response = request.data(req_ctx.clone()).execute(&server_ctx.schema).await;

  if server_ctx.server.enable_cache_control() {
    let ttl = crate::http::min_ttl(req_ctx.get_cached_values().values());
    response = response.set_cache_control(ttl);
  }

  response.to_response()
}
fn not_found() -> Result<Response<Body>> {
  Ok(Response::builder().status(StatusCode::NOT_FOUND).body(Body::empty())?)
}

async fn handle_request(req: Request<Body>, state: Arc<RwLock<ServerContext>>) -> Result<Response<Body>> {
  let server_ctx;
  {
    let state = state.read().unwrap();
    server_ctx = state.clone(); // Clone the ServerContext here
    match *req.method() {
      hyper::Method::GET if server_ctx.server.enable_graphiql.as_ref() == Some(&req.uri().path().to_string()) => {
        return graphiql()
      }
      hyper::Method::POST if req.uri().path() == "/graphql" => (),
      _ => return not_found(),
    }
  }
  // Now the lock is dropped, and we can await
  graphql_request(req, &server_ctx).await
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
  let mut etag: Option<String> = None;

  let server_sdl = if file_path.starts_with("http://") || file_path.starts_with("https://") {
    let resp = reqwest::get(file_path).await?;

    if let Some(etag_value) = resp.headers().get(reqwest::header::ETAG) {
      etag = Some(etag_value.to_str()?.to_string());
    }
    resp.text().await?
  } else {
    fs::read_to_string(file_path)?
  };

  let config = Config::from_sdl(&server_sdl)?;

  let port = config.port();
  let server = config.server.clone();
  let blueprint = Blueprint::try_from(&config).map_err(CLIError::from)?;
  let state = Arc::new(RwLock::new(ServerContext::new(blueprint, server)));

  let state_clone = Arc::clone(&state);

  let make_svc = make_service_fn(move |_conn| {
    let state = Arc::clone(&state);
    async move { Ok::<_, anyhow::Error>(service_fn(move |req| handle_request(req, state.clone()))) }
  });

  let addr: SocketAddr = ([0, 0, 0, 0], port).into();

  let server = hyper::Server::try_bind(&addr).map_err(CLIError::from)?.serve(make_svc);

  log::info!("🚀 Tailcall launched at [{}]", addr);
  if let Some(graphiql) = config.server.enable_graphiql.as_ref() {
    log::info!("🌍 Playground: http://{}{}", addr, graphiql);
  }

  let refresh_interval = 10;
  let client = reqwest::Client::new();
  let file_path_clone = file_path.clone();

  let mut interval = time::interval(Duration::from_secs(refresh_interval));
  tokio::spawn(async move {
    loop {
      interval.tick().await;

      let mut headers = HashMap::new();

      if let Some(etag_value) = &etag {
        headers.insert(IF_NONE_MATCH, HeaderValue::from_str(etag_value).unwrap());
      }

      let mut resp = client.get(&file_path_clone);

      for (k, v) in headers {
        resp = resp.header(k, v);
      }

      let resp = resp.send().await;

      let resp = match resp {
        Ok(resp) => resp,
        Err(e) => {
          log::error!("Failed to refresh configuration: {}", e);
          continue;
        }
      };

      if resp.status() == 304 {
        log::info!("The resource has not been modified.");
        continue;
      }

      if !resp.status().is_success() {
        log::info!("Unknown error.");
        continue;
      }

      if let Some(new_etag) = resp.headers().get("etag") {
        etag = Some(new_etag.to_str().unwrap().to_string());
      }

      let updated_sdl = match resp.text().await {
        Ok(updated_sdl) => updated_sdl,
        Err(_) => continue,
      };

      match Config::from_sdl(&updated_sdl) {
        Ok(updated_config) => {
          println!("{:?}", updated_config);
          let mut state = state_clone.write().unwrap();
          match Blueprint::try_from(&updated_config) {
            Ok(blueprint) => {
              state.schema = blueprint.to_schema(&state.server);
              state.server = updated_config.server;
            }
            Err(e) => {
              log::error!("Failed to create blueprint: {}", e);
              continue;
            }
          }
        }
        Err(_) => continue,
      };
    }
  });

  Ok(server.await.map_err(CLIError::from)?)
}
