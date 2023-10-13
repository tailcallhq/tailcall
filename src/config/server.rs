use std::collections::HashSet;

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
  pub headers: Vec<String>,
}
impl Default for Batch {
  fn default() -> Self {
    Batch { max_size: 1000, delay: 0, headers: Vec::new() }
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
  pub allowed_headers: Option<HashSet<String>>,
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
  pub fn get_allowed_headers(&self) -> HashSet<String> {
    self.allowed_headers.clone().unwrap_or_default()
  }
}
