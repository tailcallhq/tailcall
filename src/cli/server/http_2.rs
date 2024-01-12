#![allow(clippy::too_many_arguments)]
use std::io::BufReader;
use std::sync::Arc;

use anyhow::Result;
use hyper::server::conn::AddrIncoming;
use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use hyper_rustls::TlsAcceptor;
use rustls_pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs1KeyDer, PrivatePkcs8KeyDer, PrivateSec1KeyDer};
use tokio::fs::File;
use tokio::sync::oneshot;

use super::server_config::ServerConfig;
use crate::async_graphql_hyper::{GraphQLBatchRequest, GraphQLRequest};
use crate::cli::CLIError;
use crate::http::handle_request;

async fn load_cert(filename: String) -> Result<Vec<CertificateDer<'static>>, std::io::Error> {
  let file = File::open(filename).await?;
  let file = file.into_std().await;
  let mut file = BufReader::new(file);

  let certificates = rustls_pemfile::certs(&mut file)?;

  Ok(certificates.into_iter().map(CertificateDer::from).collect())
}

async fn load_private_key(filename: String) -> anyhow::Result<PrivateKeyDer<'static>> {
  let file = File::open(filename).await?;
  let file = file.into_std().await;
  let mut file = BufReader::new(file);

  let keys = rustls_pemfile::read_all(&mut file)?;

  if keys.len() != 1 {
    return Err(CLIError::new("Expected a single private key").into());
  }

  let key = keys.into_iter().find_map(|key| match key {
    rustls_pemfile::Item::RSAKey(key) => Some(PrivateKeyDer::Pkcs1(PrivatePkcs1KeyDer::from(key))),
    rustls_pemfile::Item::ECKey(key) => Some(PrivateKeyDer::Sec1(PrivateSec1KeyDer::from(key))),
    rustls_pemfile::Item::PKCS8Key(key) => Some(PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(key))),
    _ => None,
  });

  key.ok_or(CLIError::new("Invalid private key").into())
}

pub async fn start_http_2(
  sc: Arc<ServerConfig>,
  cert: String,
  key: String,
  server_up_sender: Option<oneshot::Sender<()>>,
) -> anyhow::Result<()> {
  let addr = sc.addr();
  let cert_chain = load_cert(cert).await?;
  let key = load_private_key(key).await?;
  let incoming = AddrIncoming::bind(&addr)?;
  let acceptor = TlsAcceptor::builder()
    .with_single_cert(cert_chain, key)?
    .with_http2_alpn()
    .with_incoming(incoming);
  let make_svc_single_req = make_service_fn(|_conn| {
    let state = Arc::clone(&sc);
    async move {
      Ok::<_, anyhow::Error>(service_fn(move |req| {
        handle_request::<GraphQLRequest>(req, state.server_context.clone())
      }))
    }
  });

  let make_svc_batch_req = make_service_fn(|_conn| {
    let state = Arc::clone(&sc);
    async move {
      Ok::<_, anyhow::Error>(service_fn(move |req| {
        handle_request::<GraphQLBatchRequest>(req, state.server_context.clone())
      }))
    }
  });

  let builder = Server::builder(acceptor).http2_only(true);

  super::log_launch_and_open_browser(sc.as_ref());

  if let Some(sender) = server_up_sender {
    sender.send(()).or(Err(anyhow::anyhow!("Failed to send message")))?;
  }

  let server: std::prelude::v1::Result<(), hyper::Error> = if sc.blueprint.server.enable_batch_requests {
    builder.serve(make_svc_batch_req).await
  } else {
    builder.serve(make_svc_single_req).await
  };

  let result = server.map_err(CLIError::from);

  Ok(result?)
}
