use std::collections::HashMap;
use std::sync::Arc;

use async_graphql::dataloader::{DataLoader, NoCache};
use derive_setters::Setters;
use hyper::HeaderMap;

use super::{DataLoaderRequest, DefaultHttpClient, HttpDataLoader, Response, ServerContext};
use crate::config::Server;

#[derive(Setters)]
pub struct RequestContext {
  pub http_client: DefaultHttpClient,
  pub server: Server,
  pub data_loader: Arc<DataLoader<HttpDataLoader<DefaultHttpClient>, NoCache>>,
  pub req_headers: HeaderMap,
}

impl Default for RequestContext {
  fn default() -> Self {
    RequestContext::new(
      DefaultHttpClient::default(),
      Server::default(),
      Arc::new(DataLoader::new(
        HttpDataLoader::new(DefaultHttpClient::default()),
        tokio::spawn,
      )),
    )
  }
}

impl RequestContext {
  pub fn new(
    http_client: DefaultHttpClient,
    server: Server,
    data_loader: Arc<DataLoader<HttpDataLoader<DefaultHttpClient>, NoCache>>,
  ) -> Self {
    Self { req_headers: HeaderMap::new(), http_client: http_client.clone(), server: server.clone(), data_loader }
  }

  #[allow(clippy::mutable_key_type)]
  pub fn get_cached_values(&self) -> HashMap<DataLoaderRequest, Response> {
    #[allow(clippy::mutable_key_type)]
    self.data_loader.get_cached_values()
  }

  pub async fn execute(&self, req: reqwest::Request) -> anyhow::Result<Response> {
    Ok(self.http_client.execute(req).await?)
  }
}

impl From<&ServerContext> for RequestContext {
  fn from(server_ctx: &ServerContext) -> Self {
    let http_client = server_ctx.http_client.clone();
    Self::new(http_client, server_ctx.server.clone(), server_ctx.data_loader.clone())
  }
}
