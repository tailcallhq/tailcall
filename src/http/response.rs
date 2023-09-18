use derive_setters::Setters;

use http_cache_semantics::ResponseLike;

use anyhow::Result;

#[derive(Clone, Debug, Default, Setters)]
pub struct Response {
  pub status: reqwest::StatusCode,
  pub headers: reqwest::header::HeaderMap,
  pub body: async_graphql::Value,
}

impl Response {
  pub async fn from_response(resp: reqwest::Response) -> Result<Self> {
    let status = resp.status();
    let headers = resp.headers().to_owned();
    let body = resp.bytes().await?;
    let json = serde_json::from_slice(&body)?;
    Ok(Response { status, headers, body: json })
  }
}

impl ResponseLike for Response {
  fn status(&self) -> reqwest::StatusCode {
    self.status
  }

  fn headers(&self) -> &reqwest::header::HeaderMap {
    &self.headers
  }
}
