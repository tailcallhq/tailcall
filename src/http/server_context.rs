use async_graphql::dynamic;
use derive_setters::Setters;

use crate::blueprint::Blueprint;
use crate::config::Server;
use crate::http::HttpClient;

#[derive(Clone, Setters)]
pub struct ServerContext {
  pub schema: dynamic::Schema,
  pub http_client: HttpClient,
  pub server: Server,
}

impl ServerContext {
  pub fn new(blueprint: Blueprint, server: Server) -> Self {
    ServerContext { schema: blueprint.to_schema(&server), http_client: HttpClient::new(server.clone()), server }
  }
}
