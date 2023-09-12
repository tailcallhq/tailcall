use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Server {
    pub allowed_headers: Option<Vec<String>>,
    #[serde(rename = "baseURL", serialize_with = "super::url::serialize")]
    pub base_url: Option<url::Url>,
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
    pub vars: Option<BTreeMap<String, String>>,
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
        !self.enable_query_validation.unwrap_or(true)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Proxy {
    pub url: String,
}
