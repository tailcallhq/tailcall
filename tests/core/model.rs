use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use tailcall::http::Method;
use url::Url;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Annotation {
    Skip,
    Only,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct UpstreamRequest(pub APIRequest);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UpstreamResponse(pub APIResponse);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Mock {
    pub request: UpstreamRequest,
    pub response: UpstreamResponse,
    #[serde(default = "default_expected_hits")]
    pub expected_hits: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct APIRequest {
    #[serde(default)]
    pub method: Method,
    pub url: Url,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    #[serde(default)]
    pub body: serde_json::Value,
    #[serde(default)]
    pub test_traces: bool,
    #[serde(default)]
    pub test_metrics: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct APIResponse {
    #[serde(default = "default_status")]
    pub status: u16,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    #[serde(default)]
    pub body: serde_json::Value,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_body: Option<String>,
}

fn default_status() -> u16 {
    200
}

fn default_expected_hits() -> usize {
    1
}
