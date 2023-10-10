use std::collections::{HashMap, HashSet};
use std::fs;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use async_graphql::http::GraphiQLSource;
use hyper::header::{HeaderValue, IF_NONE_MATCH};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, HeaderMap, Request, Response, StatusCode};
use socket2::{Domain, Socket, Type};
use std::net::TcpListener;
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
  let allowed = server.upstream.allowed_headers.unwrap_or_default();
  let headers = create_allowed_headers(req.headers(), &allowed);
  let bytes = hyper::body::to_bytes(req.into_body()).await?;
  let request: async_graphql_hyper::GraphQLRequest = serde_json::from_slice(&bytes)?;
  let req_ctx = Arc::new(RequestContext::from(server_ctx).req_headers(headers));
  let mut response = request.data(req_ctx.clone()).execute(&server_ctx.schema).await;

  if server_ctx.server.enable_cache_control() {
    if let Some(ttl) = req_ctx.get_min_max_age() {
      response = response.set_cache_control(ttl as i32);
    }
  }

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
  let state = Arc::new(ServerContext::new(blueprint, server));
  let make_svc = make_service_fn(move |_conn| {
    let state = Arc::clone(&state);
    async move { Ok::<_, anyhow::Error>(service_fn(move |req| handle_request(req, state.clone()))) }
  });

  let addr: SocketAddr = ([0, 0, 0, 0], port).into();

  let socket = Socket::new(Domain::IPV4, Type::STREAM, None).map_err(|e| {
    log::error!("Failed to create socket: {}", e);
    e
  })?;

  if let Err(e) = socket.set_reuse_port(true) {
    log::error!("Failed to set reuse port: {}", e);
  }
  if let Err(e) = socket.bind(&addr.into()) {
    log::error!("Failed to bind socket: {}", e);
  }
  if let Err(e) = socket.listen(128) {
    log::error!("Failed to listen on socket: {}", e);
  }

  let listener: TcpListener = socket.into();

  let server = hyper::Server::from_tcp(listener)
    .map_err(CLIError::from)?
    .serve(make_svc);

  log::info!("🚀 Tailcall launched at [{}]", addr);
  if let Some(graphiql) = config.server.enable_graphiql.as_ref() {
    log::info!("🌍 Playground: http://{}{}", addr, graphiql);
  }

  let refresh_interval = 10; // default to 5 minutes
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
          let port = updated_config.port();
          let server = updated_config.server.clone();

          let blueprint = match Blueprint::try_from(&updated_config) {
            Ok(blueprint) => blueprint,
            Err(e) => {
              log::error!("Failed to create blueprint: {}", e);
              continue;
            }
          };

          let state = Arc::new(ServerContext::new(blueprint, server));
          let make_svc = make_service_fn(move |_conn| {
            let state = Arc::clone(&state);
            async move { Ok::<_, anyhow::Error>(service_fn(move |req| handle_request(req, state.clone()))) }
          });

          let addr: SocketAddr = ([0, 0, 0, 0], port).into();
          let socket = match Socket::new(Domain::IPV4, Type::STREAM, None) {
            Ok(socket) => socket,
            Err(e) => {
              log::error!("Failed to create socket: {}", e);
              continue;
            }
          };

          if let Err(e) = socket.set_reuse_port(true) {
            log::error!("Failed to set reuse port: {}", e);
            continue;
          }
          if let Err(e) = socket.set_reuse_address(true) {
            log::error!("Failed to set reuse address: {}", e);
            continue;
          }
          if let Err(e) = socket.bind(&addr.into()) {
            log::error!("Failed to bind socket: {}", e);
            continue;
          }
          if let Err(e) = socket.listen(999999) {
            log::error!("Failed to listen on socket: {}", e);
            continue;
          }

          let listener: TcpListener = socket.into();
          match hyper::Server::from_tcp(listener) {
            Ok(server) => {
              log::info!("server reloaded");
              server.serve(make_svc);
            }
            Err(e) => {
              log::error!("Failed to bind server: {}", e);
              continue;
            }
          };
        }
        Err(_) => continue,
      };
    }
  });

  Ok(server.await.map_err(CLIError::from)?)
}