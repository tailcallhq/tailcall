use derive_setters::Setters;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use tailcall::config::Config;

use crate::common::{Annotation, DownstreamRequest, DownstreamResponse, Mock, UpstreamRequest};

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
    pub env: IndexMap<String, String>,

    #[serde(default)]
    pub expected_upstream_requests: Vec<UpstreamRequest>,
    pub assert: Vec<DownstreamAssertion>,

    // Annotations for the runner
    pub runner: Option<Annotation>,
}
