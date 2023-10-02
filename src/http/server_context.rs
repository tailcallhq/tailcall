use std::sync::Arc;

use async_graphql::dataloader::{DataLoader, NoCache};
use async_graphql::dynamic;
use derive_setters::Setters;

use crate::blueprint::Blueprint;
use crate::config::Server;
use crate::http::{HttpClient, HttpDataLoader};

#[derive(Setters, Clone)]
pub struct ServerContext {
  pub schema: dynamic::Schema,
  pub http_client: HttpClient,
  pub server: Server,
  pub data_loader: Arc<DataLoader<HttpDataLoader, NoCache>>,
}

impl ServerContext {
  pub fn new(blueprint: Blueprint, server: Server) -> Self {
    let schema = blueprint.to_schema(&server);
    let http_client = HttpClient::new(server.clone());
    let data_loader = HttpDataLoader::new(http_client.clone()).to_async_data_loader_options(
      server.batch.as_ref().map(|b| b.delay).unwrap_or(0),
      server.batch.as_ref().map(|b| b.max_size).unwrap_or(1000),
    );
    ServerContext { schema, http_client, server: server.clone(), data_loader: Arc::new(data_loader) }
  }
}
