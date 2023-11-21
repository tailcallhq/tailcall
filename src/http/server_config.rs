use std::net::SocketAddr;
use std::sync::Arc;

use client::DefaultHttpClient;

use super::ServerContext;
use crate::blueprint::{Blueprint, Http};
use crate::http::client;

pub struct ServerConfig {
  pub blueprint: Blueprint,
  pub server_context: Arc<ServerContext>,
}

impl ServerConfig {
  pub fn new(blueprint: Blueprint) -> Self {
    let http_client = Arc::new(DefaultHttpClient::new(&blueprint.upstream));
    Self { server_context: Arc::new(ServerContext::new(blueprint.clone(), http_client)), blueprint }
  }

  pub fn addr(&self) -> SocketAddr {
    (self.blueprint.server.hostname, self.blueprint.server.port).into()
  }

  pub fn workers(&self) -> usize {
    self.blueprint.server.worker
  }

  pub fn tokio_runtime(&self) -> anyhow::Result<tokio::runtime::Runtime> {
    let workers = self.workers();

    Ok(
      tokio::runtime::Builder::new_multi_thread()
        .worker_threads(workers)
        .enable_all()
        .build()?,
    )
  }

  pub fn http_version(&self) -> String {
    match self.blueprint.server.http {
      Http::HTTP2 { cert: _, key: _ } => "HTTP/2".to_string(),
      _ => "HTTP/1.1".to_string(),
    }
  }

  pub fn graphiql(&self) -> bool {
    self.blueprint.server.enable_graphiql
  }
}
