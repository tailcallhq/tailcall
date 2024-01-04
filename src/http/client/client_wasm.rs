use std::time::Duration;

use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};

use crate::config;
use crate::config::Upstream;
use crate::grpc::protobuf::ProtobufOperation;
use crate::http::{HttpClient, Response};

#[async_trait::async_trait]
impl HttpClient for DefaultHttpClient {
  async fn execute(&self, req: reqwest::Request, operation: Option<ProtobufOperation>) -> anyhow::Result<Response> {
    async_std::task::spawn_local(execute(self.client.clone(), req, operation)).await
  }
}

#[derive(Clone)]
pub struct DefaultHttpClient {
  client: ClientWithMiddleware,
}

impl Default for DefaultHttpClient {
  fn default() -> Self {
    let upstream = Upstream::default();
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

  pub fn with_options(_: &Upstream, _: HttpClientOptions) -> Self {
    let builder = Client::builder();
    let client = ClientBuilder::new(builder.build().expect("Failed to build client"));
    DefaultHttpClient { client: client.build() }
  }
}

async fn execute(
  client: ClientWithMiddleware,
  req: reqwest::Request,
  operation: Option<ProtobufOperation>,
) -> anyhow::Result<Response> {
  let response = client.execute(req).await?.error_for_status()?;
  let response = Response::from_response(response, operation).await?;
  Ok(response)
}
