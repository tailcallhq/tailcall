use std::collections::{BTreeMap, HashSet};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Server {
  pub allowed_headers: Option<HashSet<String>>,
  #[serde(rename = "baseURL")]
  pub base_url: Option<String>,
  pub enable_apollo_tracing: Option<bool>,
  pub enable_cache_control_header: Option<bool>,
  pub enable_graphiql: Option<String>,
  pub enable_http_cache: Option<bool>,
  pub enable_introspection: Option<bool>,
  pub enable_query_validation: Option<bool>,
  pub enable_response_validation: Option<bool>,
  pub global_response_timeout: Option<i64>,
  pub port: Option<u16>,
  pub vars: Option<BTreeMap<String, String>>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub upstream: Option<Upstream>,
}

impl Server {
  pub fn enable_http_cache(&self) -> bool {
    self.enable_http_cache.unwrap_or(false)
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
  pub fn allowed_headers(&self) -> HashSet<String> {
    // TODO: cloning isn't required we can return a ref here
    self.allowed_headers.clone().unwrap_or_default()
  }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Proxy {
  pub url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Upstream {
  #[serde(default = "pool_idle_timeout_default")]
  pub pool_idle_timeout: u64,
  #[serde(default = "pool_max_idle_per_host_default")]
  pub pool_max_idle_per_host: usize,
  #[serde(default = "keep_alive_interval_default")]
  pub keep_alive_interval: u64,
  #[serde(default = "keep_alive_timeout_default")]
  pub keep_alive_timeout: u64,
  #[serde(default = "keep_alive_while_idle_default")]
  pub keep_alive_while_idle: bool,
  #[serde(default = "proxy_default")]
  pub proxy: Option<Proxy>,
  #[serde(default = "connect_timeout_default")]
  pub connect_timeout: u64,
  #[serde(default = "timeout_default")]
  pub timeout: u64,
  #[serde(default = "tcp_keep_alive_default")]
  pub tcp_keep_alive: u64,
  #[serde(default = "user_agent_default")]
  pub user_agent: String,
}

impl Default for Upstream {
  fn default() -> Self {
    Upstream {
      pool_idle_timeout: pool_idle_timeout_default(),
      pool_max_idle_per_host: pool_max_idle_per_host_default(),
      keep_alive_interval: keep_alive_interval_default(),
      keep_alive_timeout: keep_alive_timeout_default(),
      keep_alive_while_idle: keep_alive_while_idle_default(),
      proxy: proxy_default(),
      connect_timeout: connect_timeout_default(),
      timeout: timeout_default(),
      tcp_keep_alive: tcp_keep_alive_default(),
      user_agent: user_agent_default(),
    }
  }
}

fn pool_idle_timeout_default() -> u64 {
  60
}

fn pool_max_idle_per_host_default() -> usize {
  200
}

fn keep_alive_interval_default() -> u64 {
  60
}

fn keep_alive_timeout_default() -> u64 {
  60
}

fn keep_alive_while_idle_default() -> bool {
  false
}

fn proxy_default() -> Option<Proxy> {
  None
}

fn connect_timeout_default() -> u64 {
  60
}

fn timeout_default() -> u64 {
  60
}

fn tcp_keep_alive_default() -> u64 {
  5
}

fn user_agent_default() -> String {
  "Tailcall/1.0".to_string()
}
