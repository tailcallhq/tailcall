use reqwest::Request;
use reqwest_middleware::ClientWithMiddleware;

use super::Response;
use crate::config::{self, Upstream};

#[async_trait::async_trait]
pub trait HttpClient: Sync + Send {
  async fn execute(&self, req: reqwest::Request) -> anyhow::Result<Response<async_graphql::Value>>;
  async fn execute_raw(&self, req: reqwest::Request) -> anyhow::Result<Response<Vec<u8>>>;
}

#[async_trait::async_trait]
impl HttpClient for DefaultHttpClient {
  async fn execute(&self, request: reqwest::Request) -> anyhow::Result<Response<async_graphql::Value>> {
    crate::io::http::execute(&self.client, request).await
  }

  async fn execute_raw(&self, request: Request) -> anyhow::Result<Response<Vec<u8>>> {
    crate::io::http::execute_raw(&self.client, request).await
  }
}

#[derive(Clone)]
pub struct DefaultHttpClient {
  client: ClientWithMiddleware,
}

impl Default for DefaultHttpClient {
  fn default() -> Self {
    let upstream = config::Upstream::default();
    //TODO: default is used only in tests. Drop default and move it to test.
    DefaultHttpClient::new(&upstream)
  }
}

#[derive(Default)]
pub struct HttpClientOptions {
  pub http2_only: bool,
}

impl DefaultHttpClient {
  pub fn new(upstream: &Upstream) -> Self {
    Self::with_options(upstream, HttpClientOptions::default())
  }
  pub fn with_options(upstream: &Upstream, options: HttpClientOptions) -> Self {
    DefaultHttpClient { client: crate::io::http::make_client(upstream, options) }
  }
}