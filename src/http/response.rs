use std::time::SystemTime;

use derive_setters::Setters;

use http_cache_semantics::{CachePolicy, RequestLike, ResponseLike};

use super::stats::Stats;

use anyhow::Result;

#[derive(Clone, Debug, Default, Setters)]
pub struct Response {
  pub status: reqwest::StatusCode,
  pub headers: reqwest::header::HeaderMap,
  pub body: async_graphql::Value,
  pub stats: Stats,
}

impl Response {
  pub fn min_ttl(mut self, value: u64) -> Self {
    self.stats.min_ttl = Some(value);
    self
  }

  pub async fn from_response(resp: reqwest::Response) -> Result<Self> {
    let status = resp.status();
    let headers = resp.headers().to_owned();
    let body = resp.bytes().await?;
    let json = serde_json::from_slice(&body)?;
    Ok(Response { status, headers, body: json, stats: Stats::default() })
  }

  pub fn set_min_ttl<R: RequestLike>(self, req: R) -> Self {
    let value = CachePolicy::new(&req, &self).time_to_live(SystemTime::now()).as_secs();
    self.min_ttl(value)
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
