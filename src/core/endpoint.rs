use derive_setters::Setters;
use http::header::HeaderMap;

use crate::core::config::Encoding;
use crate::core::http::Method;
use crate::core::json::JsonSchema;

#[derive(Clone, Debug, Setters)]
pub struct Endpoint {
    pub path: String,
    pub query: Vec<(String, String, bool)>,
    pub method: Method,
    pub input: JsonSchema,
    pub output: JsonSchema,
    pub headers: HeaderMap,
    pub body: Option<serde_json::Value>,
    pub description: Option<String>,
    pub encoding: Encoding,
}

impl Endpoint {
    pub fn new(url: String) -> Endpoint {
        Self {
            path: url,
            query: Default::default(),
            method: Default::default(),
            input: Default::default(),
            output: Default::default(),
            headers: Default::default(),
            body: Default::default(),
            description: Default::default(),
            encoding: Default::default(),
        }
    }
}
