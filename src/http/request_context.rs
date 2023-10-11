use std::sync::Arc;

use derive_setters::Setters;
use hyper::HeaderMap;
use tokio::sync::Mutex;

use super::{DefaultHttpClient, Response, ServerContext};
use crate::config::Server;

#[derive(Setters)]
pub struct RequestContext {
  pub http_client: DefaultHttpClient,
  pub server: Server,
  pub req_headers: HeaderMap,
  pub min_max_age: Arc<Mutex<Option<u64>>>,
}

impl Default for RequestContext {
  fn default() -> Self {
    RequestContext::new(DefaultHttpClient::default(), Server::default())
  }
}

impl RequestContext {
  pub fn new(http_client: DefaultHttpClient, server: Server) -> Self {
    Self { req_headers: HeaderMap::new(), http_client, server, min_max_age: Arc::new(Mutex::new(None)) }
  }

  pub async fn execute(&self, req: reqwest::Request) -> anyhow::Result<Response> {
    Ok(self.http_client.execute(req).await?)
  }
}

impl From<&ServerContext> for RequestContext {
  fn from(server_ctx: &ServerContext) -> Self {
    let http_client = server_ctx.http_client.clone();
    Self::new(http_client, server_ctx.server.clone())
  }
}
