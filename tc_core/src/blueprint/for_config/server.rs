use std::collections::{BTreeMap, BTreeSet};
use std::net::IpAddr;

use derive_setters::Setters;
use hyper::HeaderMap;
use serde::{Deserialize, Serialize};

use crate::blueprint::is_default;

#[derive(Clone, Debug, Setters)]
pub struct Server {
  pub enable_apollo_tracing: bool,
  pub enable_cache_control_header: bool,
  pub enable_graphiql: bool,
  pub enable_introspection: bool,
  pub enable_query_validation: bool,
  pub enable_response_validation: bool,
  pub enable_batch_requests: bool,
  pub global_response_timeout: i64,
  pub worker: usize,
  pub port: u16,
  pub hostname: IpAddr,
  pub vars: BTreeMap<String, String>,
  pub response_headers: HeaderMap,
  pub http: Http,
  pub pipeline_flush: bool,
}

#[derive(Clone, Debug)]
pub enum Http {
  HTTP1,
  HTTP2 { cert: String, key: String },
}

impl Default for Server {
  fn default() -> Self {
    // unimplemented!()
    // NOTE: Using unwrap because try_from default will never fail
    // Server::try_from(config::Server::default()).unwrap()
    Server {
      enable_apollo_tracing: false,
      enable_cache_control_header: false,
      enable_graphiql: false,
      enable_introspection: false,
      enable_query_validation: false,
      enable_response_validation: false,
      enable_batch_requests: false,
      global_response_timeout: 0,
      worker: 0,
      port: 0,
      hostname: IpAddr::V4("127.0.0.1".parse().unwrap()),
      vars: Default::default(),
      response_headers: Default::default(),
      http: Http::HTTP1,
      pipeline_flush: false,
    }
  }
}

impl Server {
  pub fn get_enable_http_validation(&self) -> bool {
    self.enable_response_validation
  }
  pub fn get_enable_cache_control(&self) -> bool {
    self.enable_cache_control_header
  }

  pub fn get_enable_introspection(&self) -> bool {
    self.enable_introspection
  }

  pub fn get_enable_query_validation(&self) -> bool {
    self.enable_query_validation
  }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct Proxy {
  pub url: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone, Default)]
pub enum HttpVersion {
  #[default]
  HTTP1,
  HTTP2,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, Setters)]
#[serde(rename_all = "camelCase", default)]
pub struct Batch {
  pub max_size: usize,
  pub delay: usize,
  pub headers: BTreeSet<String>,
}
impl Default for Batch {
  fn default() -> Self {
    Batch { max_size: 100, delay: 1, headers: BTreeSet::new() }
  }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, Setters, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Upstream {
  #[serde(default, skip_serializing_if = "is_default")]
  pub pool_idle_timeout: Option<u64>,
  #[serde(default, skip_serializing_if = "is_default")]
  pub pool_max_idle_per_host: Option<usize>,
  #[serde(default, skip_serializing_if = "is_default")]
  pub keep_alive_interval: Option<u64>,
  #[serde(default, skip_serializing_if = "is_default")]
  pub keep_alive_timeout: Option<u64>,
  #[serde(default, skip_serializing_if = "is_default")]
  pub keep_alive_while_idle: Option<bool>,
  #[serde(default, skip_serializing_if = "is_default")]
  pub proxy: Option<Proxy>,
  #[serde(default, skip_serializing_if = "is_default")]
  pub connect_timeout: Option<u64>,
  #[serde(default, skip_serializing_if = "is_default")]
  pub timeout: Option<u64>,
  #[serde(default, skip_serializing_if = "is_default")]
  pub tcp_keep_alive: Option<u64>,
  #[serde(default, skip_serializing_if = "is_default")]
  pub user_agent: Option<String>,
  #[serde(default, skip_serializing_if = "is_default")]
  pub allowed_headers: Option<BTreeSet<String>>,
  #[serde(rename = "baseURL", default, skip_serializing_if = "is_default")]
  pub base_url: Option<String>,
  #[serde(default, skip_serializing_if = "is_default")]
  pub http_cache: Option<bool>,
  #[serde(default, skip_serializing_if = "is_default")]
  pub batch: Option<Batch>,
}

impl Upstream {
  pub fn get_pool_idle_timeout(&self) -> u64 {
    self.pool_idle_timeout.unwrap_or(60)
  }
  pub fn get_pool_max_idle_per_host(&self) -> usize {
    self.pool_max_idle_per_host.unwrap_or(60)
  }
  pub fn get_keep_alive_interval(&self) -> u64 {
    self.keep_alive_interval.unwrap_or(60)
  }
  pub fn get_keep_alive_timeout(&self) -> u64 {
    self.keep_alive_timeout.unwrap_or(60)
  }
  pub fn get_keep_alive_while_idle(&self) -> bool {
    self.keep_alive_while_idle.unwrap_or(false)
  }
  pub fn get_connect_timeout(&self) -> u64 {
    self.connect_timeout.unwrap_or(60)
  }
  pub fn get_timeout(&self) -> u64 {
    self.timeout.unwrap_or(60)
  }
  pub fn get_tcp_keep_alive(&self) -> u64 {
    self.tcp_keep_alive.unwrap_or(5)
  }
  pub fn get_user_agent(&self) -> String {
    self.user_agent.clone().unwrap_or("Tailcall/1.0".to_string())
  }
  pub fn get_enable_http_cache(&self) -> bool {
    self.http_cache.unwrap_or(false)
  }
  pub fn get_allowed_headers(&self) -> BTreeSet<String> {
    self.allowed_headers.clone().unwrap_or_default()
  }
  pub fn get_delay(&self) -> usize {
    self.batch.clone().unwrap_or_default().delay
  }

  pub fn get_max_size(&self) -> usize {
    self.batch.clone().unwrap_or_default().max_size
  }

  pub fn merge_right(mut self, other: Self) -> Self {
    self.allowed_headers = other.allowed_headers.map(|other| {
      if let Some(mut self_headers) = self.allowed_headers {
        self_headers.extend(&mut other.iter().map(|s| s.to_owned()));
        self_headers
      } else {
        other
      }
    });
    self.base_url = other.base_url.or(self.base_url);
    self.connect_timeout = other.connect_timeout.or(self.connect_timeout);
    self.http_cache = other.http_cache.or(self.http_cache);
    self.keep_alive_interval = other.keep_alive_interval.or(self.keep_alive_interval);
    self.keep_alive_timeout = other.keep_alive_timeout.or(self.keep_alive_timeout);
    self.keep_alive_while_idle = other.keep_alive_while_idle.or(self.keep_alive_while_idle);
    self.pool_idle_timeout = other.pool_idle_timeout.or(self.pool_idle_timeout);
    self.pool_max_idle_per_host = other.pool_max_idle_per_host.or(self.pool_max_idle_per_host);
    self.proxy = other.proxy.or(self.proxy);
    self.tcp_keep_alive = other.tcp_keep_alive.or(self.tcp_keep_alive);
    self.timeout = other.timeout.or(self.timeout);
    self.user_agent = other.user_agent.or(self.user_agent);
    self.batch = other.batch.map(|other| {
      let mut batch = self.batch.unwrap_or_default();
      batch.max_size = other.max_size;
      batch.delay = other.delay;
      batch.headers.extend(other.headers);
      batch
    });
    self
  }
}

/*#[cfg(test)]
mod tests {
  use crate::config;

  #[test]
  fn test_try_from_default() {
    let actual = super::Server::try_from(config::Server::default());
    assert!(actual.is_ok())
  }
}*/
