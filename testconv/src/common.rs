use std::collections::BTreeMap;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use tailcall::http::Method;
use tailcall::valid::Cause;
use url::Url;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Annotation {
    Skip,
    Only,
    Fail,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct APIRequest {
    #[serde(default)]
    pub method: Method,
    pub url: Url,
    #[serde(default)]
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
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
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
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

#[derive(Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct SDLError {
    pub message: String,
    pub trace: Vec<String>,
    pub description: Option<String>,
}

impl<'a> From<Cause<&'a str>> for SDLError {
    fn from(value: Cause<&'a str>) -> Self {
        SDLError {
            message: value.message.to_string(),
            trace: value.trace.iter().map(|e| e.to_string()).collect(),
            description: None,
        }
    }
}

impl From<Cause<String>> for SDLError {
    fn from(value: Cause<String>) -> Self {
        SDLError {
            message: value.message.to_string(),
            trace: value.trace.iter().map(|e| e.to_string()).collect(),
            description: value.description,
        }
    }
}
