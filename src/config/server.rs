use std::collections::{HashSet};

use hyper::header::{HeaderName, HeaderValue};
use hyper::HeaderMap;
use derive_setters::Setters;
use serde::{Deserialize, Serialize};

use crate::config::{is_default, KeyValues};

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
  pub proxy: Option<Proxy>,
  #[serde(default, skip_serializing_if = "is_default")]
  pub vars: KeyValues,
  pub add_response_headers: Option<Vec<(String, String)>>,
  #[serde(skip)]
  pub response_headers: HeaderMap,
  pub batch: Option<Batch>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Setters)]
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
  pub fn set_response_headers(&mut self) {
    let mut header_map = HeaderMap::new();
    if let Some(headers) = &self.add_response_headers {
      for (k, v) in headers {
        // Unwrap is safe as invalid headers are caught in validation
        header_map.insert(
          HeaderName::from_bytes(k.as_bytes()).unwrap(),
          HeaderValue::from_str(v).unwrap(),
        );
      }
    }
    self.response_headers = header_map;
  }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Proxy {
  pub url: String,
}
