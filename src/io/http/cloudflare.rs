use anyhow::Result;
use reqwest::{Client, Request};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};

use super::HttpIO;
use crate::config::Upstream;
use crate::http::{HttpClientOptions, Response};

#[derive(Clone)]
pub struct HttpCloudflare {
  client: ClientWithMiddleware,
}

impl Default for HttpCloudflare {
  fn default() -> Self {
    Self { client: ClientBuilder::new(Client::new()).build() }
  }
}

impl HttpCloudflare {
  pub fn init(_: &Upstream, _: &HttpClientOptions) -> Self {
    let client = ClientBuilder::new(Client::new());
    Self { client: client.build() }
  }
}

#[async_trait::async_trait]
impl HttpIO for HttpCloudflare {
  async fn execute(&self, request: Request) -> Result<Response<async_graphql::Value>> {
    self.execute_raw(request).await?.to_json()
  }
  async fn execute_raw(&self, request: reqwest::Request) -> Result<Response<Vec<u8>>> {
    let client = self.client.clone();
    async_std::task::spawn_local(internal_execute_raw(client, request)).await
  }
}

async fn internal_execute_raw(client: ClientWithMiddleware, request: Request) -> Result<Response<Vec<u8>>> {
  let response = client.execute(request).await?;
  Ok(Response::from_reqwest(response).await?)
}

/*async fn internal_execute(client: ClientWithMiddleware, request: Request) -> Result<Response<Vec<u8>>> {
  let response = internal_execute_raw(client, request).await?;
  response.to_json()
}*/
