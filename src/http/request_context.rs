use std::sync::{Arc, Mutex};

use cache_control::{Cachability, CacheControl};
use derive_setters::Setters;
use hyper::HeaderMap;

use super::{DefaultHttpClient, HttpClientOptions};
use crate::blueprint::Server;
use crate::config::{self, Upstream};
use crate::data_loader::DataLoader;
use crate::graphql::GraphqlDataLoader;
use crate::grpc;
use crate::grpc::data_loader::GrpcDataLoader;
use crate::http::{DataLoaderRequest, HttpClient, HttpDataLoader, ServerContext};

#[derive(Setters)]
pub struct RequestContext {
  // TODO: consider storing http clients where they are used i.e. expression and dataloaders
  pub universal_http_client: Arc<dyn HttpClient>,
  // http2 only client is required for grpc in cases the server supports only http2
  // and the request will failed on protocol negotiation
  // having separate client for now looks like the only way to do with reqwest
  pub http2_only_client: Arc<dyn HttpClient>,
  pub server: Server,
  pub upstream: Upstream,
  pub req_headers: HeaderMap,
  pub http_data_loaders: Arc<Vec<DataLoader<DataLoaderRequest, HttpDataLoader>>>,
  pub gql_data_loaders: Arc<Vec<DataLoader<DataLoaderRequest, GraphqlDataLoader>>>,
  pub grpc_data_loaders: Arc<Vec<DataLoader<grpc::DataLoaderRequest, GrpcDataLoader>>>,
  min_max_age: Arc<Mutex<Option<i32>>>,
  cache_public: Arc<Mutex<Option<bool>>>,
}

impl Default for RequestContext {
  fn default() -> Self {
    let config::Config { server, upstream, .. } = config::Config::default();
    //TODO: default is used only in tests. Drop default and move it to test.
    let server = Server::try_from(server).unwrap();

    Self {
      req_headers: HeaderMap::new(),
      universal_http_client: Arc::new(DefaultHttpClient::new(&upstream)),
      http2_only_client: Arc::new(DefaultHttpClient::with_options(
        &upstream,
        HttpClientOptions { http2_only: true },
      )),
      server,
      upstream,
      http_data_loaders: Arc::new(vec![]),
      gql_data_loaders: Arc::new(vec![]),
      grpc_data_loaders: Arc::new(vec![]),
      min_max_age: Arc::new(Mutex::new(None)),
      cache_public: Arc::new(Mutex::new(None)),
    }
  }
}

impl RequestContext {
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

  pub fn is_batching_enabled(&self) -> bool {
    self.upstream.batch.is_some() && (self.upstream.get_delay() >= 1 || self.upstream.get_max_size() >= 1)
  }
}

impl From<&ServerContext> for RequestContext {
  fn from(server_ctx: &ServerContext) -> Self {
    Self {
      universal_http_client: server_ctx.universal_http_client.clone(),
      http2_only_client: server_ctx.http2_only_client.clone(),
      server: server_ctx.blueprint.server.clone(),
      upstream: server_ctx.blueprint.upstream.clone(),
      req_headers: HeaderMap::new(),
      http_data_loaders: server_ctx.http_data_loaders.clone(),
      gql_data_loaders: server_ctx.gql_data_loaders.clone(),
      grpc_data_loaders: server_ctx.grpc_data_loaders.clone(),
      min_max_age: Arc::new(Mutex::new(None)),
      cache_public: Arc::new(Mutex::new(None)),
    }
  }
}

#[cfg(test)]
mod test {
  

  use cache_control::Cachability;

  use crate::blueprint::Server;
  use crate::config::{self, Batch};
  use crate::http::{RequestContext};

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
    let req_ctx: RequestContext = RequestContext::default();
    req_ctx.set_cache_visibility(&Some(Cachability::Public));
    assert_eq!(req_ctx.is_cache_public(), None);
  }

  #[test]
  fn test_is_batching_enabled_default() {
    // create ctx with default batch
    let config = config::Config::default();
    let mut upstream = config.upstream.clone();
    upstream.batch = Some(Batch::default());
    let server = Server::try_from(config.server.clone()).unwrap();

    let req_ctx: RequestContext = RequestContext::default().upstream(upstream).server(server);

    assert!(req_ctx.is_batching_enabled());
  }
}
