use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tailcall::config::Config;

use derive_setters::Setters;

use crate::common::{DownstreamRequest, DownstreamResponse, Mock, UpstreamRequest};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DownstreamAssertion {
    pub request: DownstreamRequest,
    pub response: DownstreamResponse,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub enum ConfigSource {
    File(String),
    Inline(Config),
}

#[derive(Serialize, Deserialize, Clone, Setters, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HttpSpec {
    pub config: ConfigSource,
    pub name: String,
    pub description: Option<String>,

    #[serde(default)]
    pub mock: Vec<Mock>,

    #[serde(default)]
    pub env: HashMap<String, String>,

    #[serde(default)]
    pub expected_upstream_requests: Vec<UpstreamRequest>,
    pub assert: Vec<DownstreamAssertion>,
}
