use anyhow::Result;

use crate::http::{HttpClient, Response};
#[async_trait::async_trait]
pub trait HttpClientTrait {
  async fn execute(&self, req: reqwest::Request) -> Result<Response>;
}
#[async_trait::async_trait]
impl HttpClientTrait for HttpClient {
  async fn execute(&self, req: reqwest::Request) -> Result<Response> {
    let response = self.execute(req).await?;
    Ok(response)
  }
}
