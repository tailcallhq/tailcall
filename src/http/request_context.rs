use std::sync::{Arc, Mutex};

use async_graphql_value::ConstValue;
use cache_control::{Cachability, CacheControl};
use derive_setters::Setters;
use hyper::HeaderMap;

use crate::blueprint::Server;
use crate::chrono_cache::ChronoCache;
use crate::config::{self, Upstream};
use crate::data_loader::DataLoader;
use crate::graphql::GraphqlDataLoader;
use crate::http::{DataLoaderRequest, DefaultHttpClient, HttpClient, HttpDataLoader, Response, ServerContext};

#[derive(Setters)]
pub struct RequestContext {
  pub http_client: Arc<dyn HttpClient>,
  pub server: Server,
  pub upstream: Upstream,
  pub req_headers: HeaderMap,
  pub http_data_loaders: Arc<Vec<DataLoader<DataLoaderRequest, HttpDataLoader>>>,
  pub gql_data_loaders: Arc<Vec<DataLoader<DataLoaderRequest, GraphqlDataLoader>>>,
  pub cache: ChronoCache<u64, ConstValue>,
  min_max_age: Arc<Mutex<Option<i32>>>,
  cache_public: Arc<Mutex<Option<bool>>>,
}

impl Default for RequestContext {
  fn default() -> Self {
    let config = config::Config::default();
    //TODO: default is used only in tests. Drop default and move it to test.
    let server = Server::try_from(config.server.clone()).unwrap();
    RequestContext::new(Arc::new(DefaultHttpClient::default()), server, config.upstream.clone())
  }
}

impl RequestContext {
  pub fn new(http_client: Arc<dyn HttpClient>, server: Server, upstream: Upstream) -> Self {
    Self {
      req_headers: HeaderMap::new(),
      http_client,
      server,
      upstream,
      http_data_loaders: Arc::new(vec![]),
      gql_data_loaders: Arc::new(vec![]),
      cache: ChronoCache::new(),
      min_max_age: Arc::new(Mutex::new(None)),
      cache_public: Arc::new(Mutex::new(None)),
    }
  }

  pub async fn execute(&self, req: reqwest::Request) -> anyhow::Result<Response> {
    self.http_client.execute(req).await
  }
  fn set_min_max_age_conc(&self, min_max_age: i32) {
    *self.min_max_age.lock().unwrap() = Some(min_max_age);
  }
  pub fn get_min_max_age(&self) -> Option<i32> {
    *self.min_max_age.lock().unwrap()
  }

  pub fn set_cache_public_false(&self) {
    *self.cache_public.lock().unwrap() = Some(false);
  }

  pub fn is_cache_public(&self) -> Option<bool> {
    *self.cache_public.lock().unwrap()
  }

  pub fn set_min_max_age(&self, max_age: i32) {
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

  pub fn set_cache_visibility(&self, cachability: &Option<Cachability>) {
    if let Some(Cachability::Private) = cachability {
      self.set_cache_public_false()
    }
  }

  pub fn set_cache_control(&self, cache_policy: CacheControl) {
    if let Some(max_age) = cache_policy.max_age {
      self.set_min_max_age(max_age.as_secs() as i32);
    }
    self.set_cache_visibility(&cache_policy.cachability);
    if Some(Cachability::NoCache) == cache_policy.cachability {
      self.set_min_max_age(-1);
    }
  }

  pub fn cache_get(&self, key: &u64) -> Option<ConstValue> {
    self.cache.get(key)
  }

  #[allow(clippy::too_many_arguments)]
  pub fn cache_insert(&self, key: u64, value: ConstValue, ttl: u64) -> Option<ConstValue> {
    self.cache.insert(key, value, ttl)
  }
}

impl From<&ServerContext> for RequestContext {
  fn from(server_ctx: &ServerContext) -> Self {
    Self {
      http_client: server_ctx.http_client.clone(),
      server: server_ctx.blueprint.server.clone(),
      upstream: server_ctx.blueprint.upstream.clone(),
      req_headers: HeaderMap::new(),
      http_data_loaders: server_ctx.http_data_loaders.clone(),
      gql_data_loaders: server_ctx.gql_data_loaders.clone(),
      cache: ChronoCache::new(),
      min_max_age: Arc::new(Mutex::new(None)),
      cache_public: Arc::new(Mutex::new(None)),
    }
  }
}

#[cfg(test)]
mod test {

  use cache_control::Cachability;

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

  #[test]
  fn test_update_cache_visibility_private() {
    let req_ctx = RequestContext::default();
    req_ctx.set_cache_visibility(&Some(Cachability::Private));
    assert_eq!(req_ctx.is_cache_public(), Some(false));
  }

  #[test]
  fn test_update_cache_visibility_public() {
    let req_ctx = RequestContext::default();
    req_ctx.set_cache_visibility(&Some(Cachability::Public));
    assert_eq!(req_ctx.is_cache_public(), None);
  }
}
