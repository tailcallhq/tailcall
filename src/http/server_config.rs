use std::net::SocketAddr;
use std::sync::Arc;

use client::DefaultHttpClient;
use tokio::sync::oneshot;

use super::ServerContext;
use crate::blueprint::{Blueprint, Http};
use crate::http::client;

pub enum ServerMessage {
  ServerUp,
  Shutdown,
}

pub struct ServerControl {
  pub server_up: Control<ServerMessage>,
  pub shutdown: Control<ServerMessage>,
}

pub struct Control<T> {
  pub receiver: oneshot::Receiver<T>,
}

impl<T> Control<T> {
  fn new() -> (Self, oneshot::Sender<T>) {
    let (tx, rx) = oneshot::channel();
    (Self { receiver: rx }, tx)
  }
}

impl ServerControl {
  pub fn new() -> (Self, oneshot::Sender<ServerMessage>, oneshot::Sender<ServerMessage>) {
    let (server_up, server_up_sender) = Control::new();
    let (shutdown, shutdown_sender) = Control::new();

    (Self { server_up, shutdown }, server_up_sender, shutdown_sender)
  }
}

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

  pub fn http_version(&self) -> String {
    match self.blueprint.server.http {
      Http::HTTP2 { cert: _, key: _ } => "HTTP/2".to_string(),
      _ => "HTTP/1.1".to_string(),
    }
  }

  pub fn graphiql_url(&self) -> String {
    let protocol = match self.http_version().as_str() {
      "HTTP/2" => "https",
      _ => "http",
    };
    let addr = self.addr().to_string();
    format!("{}://{}", protocol, addr)
  }

  pub fn graphiql(&self) -> bool {
    self.blueprint.server.enable_graphiql
  }
}
