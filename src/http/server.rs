use std::collections::BTreeSet;
use std::convert::Infallible;
use std::io::BufReader;
use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use async_graphql::http::GraphiQLSource;
use client::DefaultHttpClient;
use hyper::server::conn::AddrIncoming;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, HeaderMap, Request, Response, Server, StatusCode};
use hyper_rustls::TlsAcceptor;
use rustls::PrivateKey;
use tokio::fs::File;

use super::request_context::RequestContext;
use super::ServerContext;
use crate::async_graphql_hyper;
use crate::blueprint::{Blueprint, Http};
use crate::cli::CLIError;
use crate::config::Config;
use crate::http::client;

fn graphiql() -> Result<Response<Body>> {
  Ok(Response::new(Body::from(
    GraphiQLSource::build()
      .title("Tailcall - GraphQL IDE")
      .endpoint("/graphql")
      .finish(),
  )))
}

pub async fn graphql_request(req: Request<Body>, server_ctx: &ServerContext) -> Result<Response<Body>> {
  let upstream = server_ctx.blueprint.upstream.clone();
  let allowed = upstream.get_allowed_headers();
  let headers = create_allowed_headers(req.headers(), &allowed);
  let bytes = hyper::body::to_bytes(req.into_body()).await?;
  let request: async_graphql_hyper::GraphQLRequest = serde_json::from_slice(&bytes)?;
  let req_ctx = Arc::new(RequestContext::from(server_ctx).req_headers(headers));
  let mut response = request.data(req_ctx.clone()).execute(&server_ctx.schema).await;
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

async fn load_cert(filename: &str) -> Result<Vec<rustls::Certificate>, std::io::Error> {
  let file = File::open(filename).await?;
  let file = file.into_std().await;
  let mut file = BufReader::new(file);

  let certificates = rustls_pemfile::certs(&mut file)?;

  Ok(certificates.into_iter().map(rustls::Certificate).collect())
}

async fn load_private_key(filename: &str) -> anyhow::Result<PrivateKey> {
  let file = File::open(filename).await?;
  let file = file.into_std().await;
  let mut file = BufReader::new(file);

  let keys = rustls_pemfile::read_all(&mut file)?;

  if keys.len() != 1 {
    return Err(CLIError::new("Expected a single private key").into());
  }

  let key = keys.into_iter().find_map(|key| match key {
    rustls_pemfile::Item::RSAKey(key) => Some(PrivateKey(key)),
    rustls_pemfile::Item::ECKey(key) => Some(PrivateKey(key)),
    rustls_pemfile::Item::PKCS8Key(key) => Some(PrivateKey(key)),
    _ => None,
  });

  key.ok_or(CLIError::new("Invalid private key").into())
}

struct ServerConfig {
  blueprint: Blueprint,
  server_context: Arc<ServerContext>,
}

impl ServerConfig {
  fn new(blueprint: Blueprint) -> Self {
    let http_client = Arc::new(DefaultHttpClient::new(&blueprint.upstream));
    Self { server_context: Arc::new(ServerContext::new(blueprint.clone(), http_client)), blueprint }
  }

  fn addr(&self) -> SocketAddr {
    (self.blueprint.server.hostname, self.blueprint.server.port).into()
  }

  fn workers(&self) -> usize {
    self.blueprint.server.worker
  }

  fn tokio_runtime(&self) -> anyhow::Result<tokio::runtime::Runtime> {
    let workers = self.workers();

    Ok(
      tokio::runtime::Builder::new_multi_thread()
        .worker_threads(workers)
        .enable_all()
        .build()?,
    )
  }

  fn http_version(&self) -> String {
    match self.blueprint.server.http {
      Http::HTTP2 { cert: _, key: _ } => "HTTP/2".to_string(),
      _ => "HTTP/1.1".to_string(),
    }
  }

  fn graphiql(&self) -> bool {
    self.blueprint.server.enable_graphiql
  }
}

pub async fn start_server(config: Config) -> Result<()> {
  let blueprint = Blueprint::try_from(&config).map_err(CLIError::from)?;
  let server_config = Arc::new(ServerConfig::new(blueprint.clone()));

  match blueprint.server.http.clone() {
    Http::HTTP2 { cert, key } => start_http_2(server_config, cert, key).await,
    Http::HTTP1 => start_http_1(server_config).await,
  }
}

async fn start_http_2(sc: Arc<ServerConfig>, cert: String, key: String) -> std::prelude::v1::Result<(), anyhow::Error> {
  let addr = sc.addr();
  let cert_chain = load_cert(&cert).await?;
  let key = load_private_key(&key).await?;
  let incoming = AddrIncoming::bind(&addr)?;
  let rt = sc.tokio_runtime()?;
  let acceptor = TlsAcceptor::builder()
    .with_single_cert(cert_chain, key)?
    .with_http2_alpn()
    .with_incoming(incoming);

  let sc_cloned = sc.clone();
  let server = Server::builder(acceptor).http2_only(true).serve(make_service_fn({
    move |_conn| {
      let sc = sc_cloned.clone();
      async move { Ok::<_, Infallible>(service_fn(move |req| handle_request(req, sc.server_context.clone()))) }
    }
  }));

  Ok(
    rt.spawn(async move {
      log_launch(sc.as_ref());
      server.await.map_err(CLIError::from)
    })
    .await??,
  )
}

fn log_launch(sc: &ServerConfig) {
  let addr = sc.addr().to_string();
  log::info!("🚀 Tailcall launched at [{}] over {}", addr, sc.http_version());
  if sc.graphiql() {
    log::info!("🌍 Playground: https://{}", addr);
  }
}

async fn start_http_1(sc: Arc<ServerConfig>) -> std::prelude::v1::Result<(), anyhow::Error> {
  let addr = sc.addr();
  let sc_cloned = sc.clone();
  Ok(
    sc.tokio_runtime()?
      .spawn(async move {
        let server = hyper::Server::try_bind(&addr)
          .map_err(CLIError::from)?
          .serve(make_service_fn(move |_conn| {
            let sc = sc_cloned.clone();
            async move { Ok::<_, Infallible>(service_fn(move |req| handle_request(req, sc.server_context.clone()))) }
          }));

        log_launch(sc.as_ref());
        server.await.map_err(CLIError::from)
      })
      .await??,
  )
}
