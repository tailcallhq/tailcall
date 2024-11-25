use std::collections::BTreeMap;
use std::panic;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tailcall::core::http::Method;
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

    pub fn concurrency() -> usize {
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
    #[serde(default)]
    pub delay: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct APIRequest {
    #[serde(default)]
    pub method: Method,
    pub url: Url,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    #[serde(flatten, default)]
    pub body: Option<APIBody>,
    #[serde(default)]
    pub test_traces: bool,
    #[serde(default)]
    pub test_metrics: bool,
    #[serde(default = "default::concurrency")]
    pub concurrency: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct APIResponse {
    #[serde(default = "default::status")]
    pub status: u16,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    #[serde(flatten, default)]
    pub body: Option<APIBody>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum APIBody {
    #[serde(rename = "textBody")]
    Text(String),
    #[serde(rename = "fileBody")]
    File(String),
    #[serde(rename = "body")]
    Value(Value),
}

impl APIBody {
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            APIBody::Value(value) => serde_json::to_vec(value)
                .unwrap_or_else(|_| core::panic!("Failed to convert value: {value:?}")),
            APIBody::Text(text) => string_to_bytes(text),
            APIBody::File(file) => {
                let path: Vec<&str> = file.rsplitn(2, '/').collect();
                match &path[..] {
                    &[file, prefix] => match prefix {
                        "grpc/reflection" => {
                            let path =
                                Path::new(tailcall_fixtures::grpc::reflection::SELF).join(file);
                            std::fs::read(&path).unwrap_or_else(|_| {
                                core::panic!("Failed to read file by path: {}", path.display())
                            })
                        }
                        _ => core::panic!("Invalid file path: {} {}", prefix, file),
                    },
                    _ => core::panic!("Invalid file path: {}", file),
                }
            }
        }
    }
}

fn string_to_bytes(input: &str) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '\\' => match chars.next() {
                Some('0') => bytes.push(0),
                Some('n') => bytes.push(b'\n'),
                Some('t') => bytes.push(b'\t'),
                Some('r') => bytes.push(b'\r'),
                Some('\\') => bytes.push(b'\\'),
                Some('\"') => bytes.push(b'\"'),
                Some('x') => {
                    let mut hex = chars.next().unwrap().to_string();
                    hex.push(chars.next().unwrap());
                    let byte = u8::from_str_radix(&hex, 16).unwrap();
                    bytes.push(byte);
                }
                _ => panic!("Unsupported escape sequence"),
            },
            _ => bytes.push(c as u8),
        }
    }

    bytes
}
