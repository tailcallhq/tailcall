use std::sync::Arc;

use async_graphql::dataloader::{DataLoader, NoCache};
use async_graphql::dynamic;
use derive_setters::Setters;
use reqwest::header::HeaderMap;

use crate::blueprint::Blueprint;
use crate::config::Server;
use crate::http::{DefaultHttpClient, HttpDataLoader};

#[derive(Setters, Clone)]
pub struct ServerContext {
  pub schema: dynamic::Schema,
  pub http_client: DefaultHttpClient,
  pub server: Server,
  pub data_loader: Arc<DataLoader<HttpDataLoader<DefaultHttpClient>, NoCache>>,
  pub response_headers: HeaderMap,
}

impl ServerContext {
  pub fn new(blueprint: Blueprint, server: Server, headers: HeaderMap) -> Self {
    let schema = blueprint.to_schema(&server);
    let http_client = DefaultHttpClient::new(server.clone());
    let data_loader = HttpDataLoader::new(http_client.clone()).to_data_loader(server.batch.clone().unwrap_or_default());
    ServerContext {
      schema,
      http_client,
      server: server.clone(),
      data_loader: Arc::new(data_loader),
      response_headers: headers,
    }
  }
}
