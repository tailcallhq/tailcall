use std::sync::{Arc, Mutex};

use derive_setters::Setters;
use hyper::HeaderMap;

use super::{DefaultHttpClient, Response, ServerContext};
use crate::blueprint::Server;
use crate::config::{self, Upstream};

#[derive(Setters)]
pub struct RequestContext {
  pub http_client: DefaultHttpClient,
  pub server: Server,
  pub upstream: Upstream,
  pub req_headers: HeaderMap,
  min_max_age: Arc<Mutex<Option<u64>>>,
}

impl Default for RequestContext {
  fn default() -> Self {
    let config = config::Config::default();
    //TODO: default is used only in tests. Drop default and move it to test.
    let server = Server::try_from(config.server.clone()).unwrap();
    RequestContext::new(DefaultHttpClient::default(), server, config.upstream.clone())
  }
}

impl RequestContext {
  pub fn new(http_client: DefaultHttpClient, server: Server, upstream: Upstream) -> Self {
    Self { req_headers: HeaderMap::new(), http_client, server, upstream, min_max_age: Arc::new(Mutex::new(None)) }
  }

  pub async fn execute(&self, req: reqwest::Request) -> anyhow::Result<Response> {
    Ok(self.http_client.execute(req).await?)
  }
  fn set_min_max_age_conc(&self, min_max_age: u64) {
    *self.min_max_age.lock().unwrap() = Some(min_max_age);
  }
  pub fn get_min_max_age(&self) -> Option<u64> {
    *self.min_max_age.lock().unwrap()
  }

  pub fn set_min_max_age(&self, max_age: u64) {
    let min_max_age_lock = self.get_min_max_age();
    match min_max_age_lock {
      Some(min_max_age) if max_age < min_max_age => {
        self.set_min_max_age_conc(max_age);
      }
      None => {
        self.set_min_max_age_conc(max_age);
      }
      _ => {}
    }
  }
}

impl From<&ServerContext> for RequestContext {
  fn from(server_ctx: &ServerContext) -> Self {
    let http_client = server_ctx.http_client.clone();
    Self::new(
      http_client,
      server_ctx.blueprint.server.clone(),
      server_ctx.blueprint.upstream.clone(),
    )
  }
}

#[cfg(test)]
mod test {

  use crate::http::RequestContext;

  #[test]
  fn test_update_max_age_less_than_existing() {
    let req_ctx = RequestContext::default();
    req_ctx.set_min_max_age(120);
    req_ctx.set_min_max_age(60);
    assert_eq!(req_ctx.get_min_max_age(), Some(60));
  }

  #[test]
  fn test_update_max_age_greater_than_existing() {
    let req_ctx = RequestContext::default();
    req_ctx.set_min_max_age(60);
    req_ctx.set_min_max_age(120);
    assert_eq!(req_ctx.get_min_max_age(), Some(60));
  }

  #[test]
  fn test_update_max_age_no_existing_value() {
    let req_ctx = RequestContext::default();
    req_ctx.set_min_max_age(120);
    assert_eq!(req_ctx.get_min_max_age(), Some(120));
  }
}
