use std::collections::HashMap;
use std::sync::Arc;

use async_graphql::dataloader::{DataLoader, NoCache};
use derive_setters::Setters;
use hyper::{HeaderMap, Uri};

use super::memo_client::MemoClient;
use super::{EndpointKey, HttpClient, HttpDataLoader, Response, ServerContext};
use crate::cache::Cache;
use crate::config::Server;

#[derive(Setters)]
pub struct RequestContext {
  pub data_loader: Arc<Option<DataLoader<HttpDataLoader, NoCache>>>,
  pub memo_client: MemoClient,
  pub http_client: HttpClient,
  pub server: Server,
  pub req_headers: HeaderMap,
  pub cache: Cache<Uri, super::Response>,
}

impl Default for RequestContext {
  fn default() -> Self {
    RequestContext::new(HttpClient::default(), Server::default(), HeaderMap::new())
  }
}

impl RequestContext {
  pub fn new(http_client: HttpClient, server: Server, headers: HeaderMap) -> Self {
    Self {
      data_loader: Arc::new(None),
      memo_client: MemoClient::new(http_client.clone()),
      req_headers: headers,
      http_client,
      server,
      cache: Cache::empty(),
    }
  }

  #[allow(clippy::mutable_key_type)]
  pub fn get_cached_values(&self) -> HashMap<EndpointKey, Response> {
    #[allow(clippy::mutable_key_type)]
    if let Some(data_loader) = &self.data_loader.as_ref() {
      data_loader.get_cached_values()
    } else {
      HashMap::new()
    }
  }

  pub async fn execute(&self, req: reqwest::Request) -> anyhow::Result<Response> {
    Ok(self.http_client.execute(req).await?)
  }
}

impl From<&ServerContext> for RequestContext {
  fn from(server_ctx: &ServerContext) -> Self {
    let http_client = server_ctx.http_client.clone();
    let server = server_ctx.server.clone();
    Self::new(http_client, server, HeaderMap::new()).data_loader(server_ctx.clone().data_loader)
  }
}
