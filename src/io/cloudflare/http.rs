use anyhow::Result;
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};

use super::HttpIO;
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
  pub fn init() -> Self {
    let client = ClientBuilder::new(Client::new());
    Self { client: client.build() }
  }
}

#[async_trait::async_trait]
impl HttpIO for HttpCloudflare {
  // HttpClientOptions are ignored in Cloudflare
  // This is because there is little control over the underlying HTTP client
  async fn execute_raw(&self, request: reqwest::Request, _: HttpClientOptions) -> Result<Response<Vec<u8>>> {
    let client = self.client.clone();
    async_std::task::spawn_local(async move {
      let response = client.execute(request).await?;
      Response::from_reqwest(response).await
    })
    .await
  }
}
