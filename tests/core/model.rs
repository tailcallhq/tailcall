use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use tailcall::http::Method;
use url::Url;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Annotation {
    Skip,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct UpstreamRequest(pub APIRequest);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UpstreamResponse(pub APIResponse);

mod default {
    pub fn status() -> u16 {
        200
    }

    pub fn expected_hits() -> usize {
        1
    }

    pub fn assert_hits() -> bool {
        true
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Mock {
    pub request: UpstreamRequest,
    pub response: UpstreamResponse,
    #[serde(default = "default::assert_hits")]
    pub assert_hits: bool,
    #[serde(default = "default::expected_hits")]
    pub expected_hits: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct APIRequest {
    #[serde(default)]
    pub method: Method,
    pub url: Url,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    #[serde(flatten, default)]
    pub body: Option<crate::core::http::ApiBody>,
    #[serde(default)]
    pub test_traces: bool,
    #[serde(default)]
    pub test_metrics: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct APIResponse {
    #[serde(default = "default::status")]
    pub status: u16,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    #[serde(flatten, default)]
    pub body: Option<crate::core::http::ApiBody>,
}
