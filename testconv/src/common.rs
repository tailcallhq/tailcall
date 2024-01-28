use std::collections::BTreeMap;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use tailcall::http::Method;
use url::Url;

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct APIRequest {
    #[serde(default)]
    pub method: Method,
    pub url: Url,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    #[serde(default)]
    pub body: serde_json::Value,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct APIResponse {
    #[serde(default = "default_status")]
    pub status: u16,
    #[serde(default)]
    pub headers: IndexMap<String, String>,
    #[serde(default)]
    pub body: serde_json::Value,
}

fn default_status() -> u16 {
    200
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct UpstreamRequest(pub APIRequest);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UpstreamResponse(pub APIResponse);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DownstreamRequest(pub APIRequest);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DownstreamResponse(pub APIResponse);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Mock {
    pub request: UpstreamRequest,
    pub response: UpstreamResponse,
}
