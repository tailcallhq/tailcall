use std::collections::HashMap;

use async_graphql::dataloader::{DataLoader, HashMapCache};
use derive_setters::Setters;
use hyper::HeaderMap;

use crate::config::Server;

use super::{memo_client::MemoClient, EndpointKey, HttpClient, HttpDataLoader, Response, ServerContext};

#[derive(Setters)]
pub struct RequestContext {
  pub data_loader: DataLoader<HttpDataLoader, HashMapCache>,
  pub memo_client: MemoClient,
  pub http_client: HttpClient,
  pub server: Server,
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
      data_loader: HttpDataLoader::new(http_client.clone()).to_async_data_loader(),
      memo_client: MemoClient::new(http_client.clone()),
      req_headers: headers,
      http_client,
      server,
    }
  }

  #[allow(clippy::mutable_key_type)]
  pub fn get_cached_values(&self) -> HashMap<EndpointKey, Response> {
    #[allow(clippy::mutable_key_type)]
    self.data_loader.get_cached_values()
  }
}

impl From<&ServerContext> for RequestContext {
  fn from(server_ctx: &ServerContext) -> Self {
    let http_client = server_ctx.http_client.clone();
    let server = server_ctx.server.clone();
    Self::new(http_client, server, HeaderMap::new())
  }
}
