use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};

use super::client::HttpClient;
use crate::http::Response;

#[async_trait::async_trait]
impl HttpClient for DefaultHttpClient {
  async fn execute(&self, request: reqwest::Request) -> anyhow::Result<Response> {
    async_std::task::spawn_local(execute(self.client.clone(), request)).await
  }

  async fn execute_raw(&self, request: reqwest::Request) -> anyhow::Result<reqwest::Response> {
    unimplemented!()
  }
}

#[derive(Clone)]
pub struct DefaultHttpClient {
  client: ClientWithMiddleware,
}

#[derive(Default)]
pub struct HttpClientOptions {
  pub http2_only: bool,
}

impl DefaultHttpClient {
  pub fn new() -> Self {
    Self::with_options()
  }

  pub fn with_options() -> Self {
    let builder = Client::builder();
    let client = ClientBuilder::new(builder.build().expect("Failed to build client"));
    DefaultHttpClient { client: client.build() }
  }
}

async fn execute(client: ClientWithMiddleware, request: reqwest::Request) -> anyhow::Result<Response> {
  let response = client.execute(request).await?;
  let response = Response::from_response(response).await?;
  Ok(response)
}
