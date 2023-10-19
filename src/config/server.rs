use std::collections::{BTreeMap, BTreeSet};

use derive_setters::Setters;
use serde::{Deserialize, Serialize};

use crate::config::{is_default, KeyValues};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Server {
  pub enable_apollo_tracing: Option<bool>,
  pub enable_cache_control_header: Option<bool>,
  pub enable_graphiql: Option<bool>,
  pub enable_introspection: Option<bool>,
  pub enable_query_validation: Option<bool>,
  pub enable_response_validation: Option<bool>,
  pub enable_batch_requests: Option<bool>,
  pub global_response_timeout: Option<i64>,
  #[serde(skip_serializing_if = "is_default")]
  pub hostname: Option<String>,
  pub port: Option<u16>,
  #[serde(default, skip_serializing_if = "is_default")]
  pub vars: KeyValues,
  #[serde(skip_serializing_if = "is_default", default)]
  pub response_headers: KeyValues,
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
    Batch { max_size: 1000, delay: 0, headers: BTreeSet::new() }
  }
}

impl Server {
  pub fn enable_apollo_tracing(&self) -> bool {
    self.enable_apollo_tracing.unwrap_or(false)
  }
  pub fn enable_graphiql(&self) -> bool {
    self.enable_graphiql.unwrap_or(false)
  }
  pub fn get_global_response_timeout(&self) -> i64 {
    self.global_response_timeout.unwrap_or(0)
  }
  pub fn get_port(&self) -> u16 {
    self.port.unwrap_or(8000)
  }
  pub fn enable_http_validation(&self) -> bool {
    self.enable_response_validation.unwrap_or(false)
  }
  pub fn enable_cache_control(&self) -> bool {
    self.enable_cache_control_header.unwrap_or(false)
  }
  pub fn enable_introspection(&self) -> bool {
    self.enable_introspection.unwrap_or(true)
  }
  pub fn enable_query_validation(&self) -> bool {
    self.enable_query_validation.unwrap_or(true)
  }
  pub fn enable_batch_requests(&self) -> bool {
    self.enable_batch_requests.unwrap_or(false)
  }

  pub fn get_hostname(&self) -> String {
    self.hostname.clone().unwrap_or("127.0.0.1".to_string())
  }

  pub fn get_vars(&self) -> BTreeMap<String, String> {
    self.vars.clone().0
  }

  pub fn get_response_headers(&self) -> KeyValues {
    self.response_headers.clone()
  }

  pub fn merge_right(mut self, other: Self) -> Self {
    self.enable_apollo_tracing = other.enable_apollo_tracing.or(self.enable_apollo_tracing);
    self.enable_cache_control_header = other.enable_cache_control_header.or(self.enable_cache_control_header);
    self.enable_graphiql = other.enable_graphiql.or(self.enable_graphiql);
    self.enable_introspection = other.enable_introspection.or(self.enable_introspection);
    self.enable_query_validation = other.enable_query_validation.or(self.enable_query_validation);
    self.enable_response_validation = other.enable_response_validation.or(self.enable_response_validation);
    self.enable_batch_requests = other.enable_batch_requests.or(self.enable_batch_requests);
    self.global_response_timeout = other.global_response_timeout.or(self.global_response_timeout);
    self.port = other.port.or(self.port);
    self.hostname = other.hostname.or(self.hostname);
    let mut vars = self.vars.0.clone();
    vars.extend(other.vars.0);
    self.vars = KeyValues(vars);
    let mut response_headers = self.response_headers.0.clone();
    response_headers.extend(other.response_headers.0);
    self.response_headers = KeyValues(response_headers);
    self
  }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct Proxy {
  pub url: String,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, Setters, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Upstream {
  #[serde(skip_serializing_if = "is_default")]
  pub pool_idle_timeout: Option<u64>,
  #[serde(skip_serializing_if = "is_default")]
  pub pool_max_idle_per_host: Option<usize>,
  #[serde(skip_serializing_if = "is_default")]
  pub keep_alive_interval: Option<u64>,
  #[serde(skip_serializing_if = "is_default")]
  pub keep_alive_timeout: Option<u64>,
  #[serde(skip_serializing_if = "is_default")]
  pub keep_alive_while_idle: Option<bool>,
  #[serde(skip_serializing_if = "is_default")]
  pub proxy: Option<Proxy>,
  #[serde(skip_serializing_if = "is_default")]
  pub connect_timeout: Option<u64>,
  #[serde(skip_serializing_if = "is_default")]
  pub timeout: Option<u64>,
  #[serde(skip_serializing_if = "is_default")]
  pub tcp_keep_alive: Option<u64>,
  #[serde(skip_serializing_if = "is_default")]
  pub user_agent: Option<String>,
  #[serde(skip_serializing_if = "is_default")]
  pub allowed_headers: Option<BTreeSet<String>>,
  #[serde(rename = "baseURL")]
  #[serde(skip_serializing_if = "is_default")]
  pub base_url: Option<String>,
  #[serde(skip_serializing_if = "is_default")]
  pub enable_http_cache: Option<bool>,
  #[serde(skip_serializing_if = "is_default")]
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
    self.enable_http_cache.unwrap_or(false)
  }
  pub fn get_allowed_headers(&self) -> BTreeSet<String> {
    self.allowed_headers.clone().unwrap_or_default()
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
    self.enable_http_cache = other.enable_http_cache.or(self.enable_http_cache);
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
