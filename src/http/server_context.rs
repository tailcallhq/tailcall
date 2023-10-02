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
  pub data_loader: Arc<Option<DataLoader<HttpDataLoader, NoCache>>>,
}

impl ServerContext {
  pub fn new(blueprint: Blueprint, server: Server) -> Self {
    let schema = blueprint.to_schema(&server);
    let http_client = HttpClient::new(server.clone());

    let mut context = ServerContext { schema, http_client, server: server.clone(), data_loader: Arc::new(None) };

    if let Some(batch) = server.batch.as_ref() {
      let data_loader = HttpDataLoader::new(context.http_client.clone())
        .to_async_data_loader_options(batch.delay.unwrap_or(0), batch.max_size.unwrap_or(1000));
      context.data_loader = Arc::new(Some(data_loader));
    }

    context
  }
}
