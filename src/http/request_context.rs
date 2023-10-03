use std::collections::HashMap;
use std::sync::Arc;

use async_graphql::dataloader::{DataLoader, NoCache};
use derive_setters::Setters;
use hyper::HeaderMap;

use super::memo_client::MemoClient;
use super::{EndpointKey, HttpClient, HttpDataLoader, Response, ServerContext};
use crate::config::Server;

#[derive(Setters)]
pub struct RequestContext {
  pub memo_client: MemoClient,
  pub http_client: HttpClient,
  pub server: Server,
  pub data_loader: Arc<DataLoader<HttpDataLoader<HttpClient>, NoCache>>,
  pub req_headers: HeaderMap,
}

impl Default for RequestContext {
  fn default() -> Self {
    RequestContext::new(HttpClient::default(), Server::default(), HeaderMap::new())
  }
}

impl RequestContext {
  pub fn new(http_client: HttpClient, server: Server, headers: HeaderMap) -> Self {
    Self {
      memo_client: MemoClient::new(http_client.clone()),
      req_headers: headers,
      http_client: http_client.clone(),
      server: server.clone(),
      data_loader: Arc::new(HttpDataLoader::new(http_client.clone()).to_async_data_loader_options(
        server.batch.clone().map(|b| b.delay).unwrap_or(0),
        server.batch.clone().map(|b| b.max_size).unwrap_or(1000),
      )),
    }
  }

  #[allow(clippy::mutable_key_type)]
  pub fn get_cached_values(&self) -> HashMap<EndpointKey, Response> {
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
    Self::new(http_client, server_ctx.server.clone(), HeaderMap::new()).data_loader(server_ctx.data_loader.clone())
  }
}
