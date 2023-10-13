use std::collections::BTreeSet;

use derive_setters::Setters;
use serde::{Deserialize, Serialize};

use crate::config::{is_default, KeyValues};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Server {
  pub enable_apollo_tracing: Option<bool>,
  pub enable_cache_control_header: Option<bool>,
  pub enable_graphiql: Option<String>,
  pub enable_introspection: Option<bool>,
  pub enable_query_validation: Option<bool>,
  pub enable_response_validation: Option<bool>,
  pub global_response_timeout: Option<i64>,
  pub port: Option<u16>,
  #[serde(default, skip_serializing_if = "is_default")]
  pub upstream: Upstream,
  #[serde(default, skip_serializing_if = "is_default")]
  pub vars: KeyValues,
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

  pub(crate) fn merge_right(self, other: Self) -> Self {
    let mut merged = self.clone();
    merged.enable_apollo_tracing = other.enable_apollo_tracing.or(self.enable_apollo_tracing);
    merged.enable_cache_control_header = other.enable_cache_control_header.or(self.enable_cache_control_header);
    merged.enable_graphiql = other.enable_graphiql.or(self.enable_graphiql);
    merged.enable_introspection = other.enable_introspection.or(self.enable_introspection);
    merged.enable_query_validation = other.enable_query_validation.or(self.enable_query_validation);
    merged.enable_response_validation = other.enable_response_validation.or(self.enable_response_validation);
    merged.global_response_timeout = other.global_response_timeout.or(self.global_response_timeout);
    merged.port = other.port.or(self.port);
    let mut vars = self.vars.0.clone();
    vars.extend(other.vars.0);
    merged.vars = KeyValues(vars);
    merged.upstream = self.upstream.merge_right(other.upstream);
    merged
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

  pub fn merge_right(self, other: Self) -> Self {
    let mut merged = self.clone();
    merged.allowed_headers = other.allowed_headers.map(|other| {
      if let Some(mut self_headers) = merged.allowed_headers {
        self_headers.extend(&mut other.iter().map(|s| s.to_owned()));
        self_headers
      } else {
        other
      }
    });
    merged.base_url = other.base_url.or(self.base_url);
    merged.connect_timeout = other.connect_timeout.or(self.connect_timeout);
    merged.enable_http_cache = other.enable_http_cache.or(self.enable_http_cache);
    merged.keep_alive_interval = other.keep_alive_interval.or(self.keep_alive_interval);
    merged.keep_alive_timeout = other.keep_alive_timeout.or(self.keep_alive_timeout);
    merged.keep_alive_while_idle = other.keep_alive_while_idle.or(self.keep_alive_while_idle);
    merged.pool_idle_timeout = other.pool_idle_timeout.or(self.pool_idle_timeout);
    merged.pool_max_idle_per_host = other.pool_max_idle_per_host.or(self.pool_max_idle_per_host);
    merged.proxy = other.proxy.or(self.proxy);
    merged.tcp_keep_alive = other.tcp_keep_alive.or(self.tcp_keep_alive);
    merged.timeout = other.timeout.or(self.timeout);
    merged.user_agent = other.user_agent.or(self.user_agent);
    merged.batch = other.batch.map(|other| {
      let mut merged = self.batch.unwrap_or_default();
      merged.max_size = other.max_size;
      merged.delay = other.delay;
      merged.headers.extend(other.headers);
      merged
    });
    merged
  }
}
