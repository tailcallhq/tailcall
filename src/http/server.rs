use std::collections::BTreeSet;
use std::convert::Infallible;
use std::fs::File;
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

fn load_cert(filename: &str) -> Result<Vec<rustls::Certificate>, std::io::Error> {
  let file = File::open(filename)?;
  let mut file = BufReader::new(file);

  let certificates = rustls_pemfile::certs(&mut file)?;

  Ok(certificates.into_iter().map(rustls::Certificate).collect())
}

fn load_private_key(filename: &str) -> Result<PrivateKey> {
  let file = File::open(filename).map_err(CLIError::from)?;
  let mut file = BufReader::new(file);

  let keys = rustls_pemfile::read_all(&mut file).map_err(CLIError::from)?;

  if keys.len() != 1 {
    return Err(CLIError::new("Expected a single private key").into());
  }

  let key = keys.into_iter().find(|key| {
    matches!(
      key,
      rustls_pemfile::Item::RSAKey(_) | rustls_pemfile::Item::ECKey(_) | rustls_pemfile::Item::PKCS8Key(_)
    )
  });

  if let Some(key) = key {
    log::info!("🔑 Loaded private key");

    Ok(match key {
      rustls_pemfile::Item::RSAKey(key) => PrivateKey(key),
      rustls_pemfile::Item::ECKey(key) => PrivateKey(key),
      rustls_pemfile::Item::PKCS8Key(key) => PrivateKey(key),
      _ => unreachable!(),
    })
  } else {
    Err(CLIError::new("No private key found").into())
  }
}

pub async fn start_server(config: Config) -> Result<()> {
  let blueprint = Blueprint::try_from(&config).map_err(CLIError::from)?;
  let http_client = Arc::new(DefaultHttpClient::new(&blueprint.upstream));
  let state = Arc::new(ServerContext::new(blueprint.clone(), http_client));
  let addr: SocketAddr = (blueprint.server.hostname, blueprint.server.port).into();

  let rt = tokio::runtime::Builder::new_multi_thread()
    .worker_threads(blueprint.server.worker)
    .enable_all()
    .build()
    .unwrap();

  match blueprint.server.http {
    Http::HTTP2 { cert, key } => {
      let make_svc = make_service_fn(move |_conn| {
        let state = Arc::clone(&state);
        async move { Ok::<_, Infallible>(service_fn(move |req| handle_request(req, state.clone()))) }
      });

      let cert_chain = load_cert(&cert).expect("Failed to load certificate");
      let key = load_private_key(&key).expect("Failed to load private key");

      let incoming = AddrIncoming::bind(&addr)?;
      let acceptor = TlsAcceptor::builder()
        .with_single_cert(cert_chain, key)
        .map_err(CLIError::from)?
        .with_http2_alpn()
        .with_incoming(incoming);

      let server = Server::builder(acceptor).http2_only(true).serve(make_svc);

      log::info!("🚀 Tailcall launched at [{}] over {}", addr, "HTTP/2.0");
      if blueprint.server.enable_graphiql {
        log::info!("🌍 Playground: https://{}", addr);
      }

      Ok(rt.spawn(async move { server.await.map_err(CLIError::from) }).await??)
    }
    Http::HTTP1 => {
      let make_svc = make_service_fn(move |_conn| {
        let state = Arc::clone(&state);
        async move { Ok::<_, Infallible>(service_fn(move |req| handle_request(req, state.clone()))) }
      });

      log::info!("🚀 Tailcall launched at [{}]", addr);
      if blueprint.server.enable_graphiql {
        log::info!("🌍 Playground: http://{}", addr);
      }

      Ok(
        rt.spawn(async move {
          let server = hyper::Server::try_bind(&addr).map_err(CLIError::from)?.serve(make_svc);
          server.await.map_err(CLIError::from)
        })
        .await??,
      )
    }
  }
}
