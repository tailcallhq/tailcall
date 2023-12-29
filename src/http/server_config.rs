use std::net::{ IpAddr, Ipv4Addr, SocketAddr };
use std::sync::Arc;

use super::ServerContext;
use crate::blueprint::{ Blueprint, Http };

pub struct ServerConfig {
  pub blueprint: Blueprint,
  pub server_context: Arc<ServerContext>,
}

impl ServerConfig {
  pub fn new(blueprint: Blueprint) -> Self {
    Self { server_context: Arc::new(ServerContext::new(blueprint.clone())), blueprint }
  }

  pub fn addr(&self) -> SocketAddr {
    (self.blueprint.server.hostname, self.blueprint.server.port).into()
  }

  pub fn http_version(&self) -> String {
    match self.blueprint.server.http {
      Http::HTTP2 { cert: _, key: _ } => "HTTP/2".to_string(),
      _ => "HTTP/1.1".to_string(),
    }
  }

  pub fn graphiql_url(&self) -> String {
    let protocol = match self.http_version().as_str() {
      "HTTP/2" => {
        match &self.blueprint.server.http {
          Http::HTTP2 { cert, key } => {
            if cert.is_empty() | key.is_empty() { "http" } else { "https" }
          }
          _ => "http",
        }
      }
      _ => "http",
    };
    let mut addr = self.addr();

    if addr.ip().is_unspecified() {
      addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), addr.port());
    }

    format!("{}://{}", protocol, addr)
  }

  pub fn graphiql(&self) -> bool {
    self.blueprint.server.enable_graphiql
  }
}
