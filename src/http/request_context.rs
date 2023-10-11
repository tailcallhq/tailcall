use std::sync::{Arc, Mutex};


use derive_setters::Setters;
use hyper::HeaderMap;

use super::{DefaultHttpClient, Response, ServerContext};
use crate::config::Server;

#[derive(Setters)]
pub struct RequestContext {
  pub http_client: DefaultHttpClient,
  pub server: Server,
  pub req_headers: HeaderMap,
  min_max_age: Arc<Mutex<Option<u64>>>,
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
    Self::new(http_client, server_ctx.server.clone())
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
