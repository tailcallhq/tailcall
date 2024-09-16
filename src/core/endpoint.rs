use derive_setters::Setters;
use hyper::HeaderMap;

use crate::core::config::Encoding;
use crate::core::http::Method;
use crate::core::json::JsonSchema;

use super::config::Proxy;

#[derive(Clone, Debug, Setters)]
pub struct Endpoint {
    pub path: String,
    pub query: Vec<(String, String, bool)>,
    pub method: Method,
    pub proxy: Option<Proxy>,
    pub input: JsonSchema,
    pub output: JsonSchema,
    pub headers: HeaderMap,
    pub body: Option<String>,
    pub description: Option<String>,
    pub encoding: Encoding,
}

impl Endpoint {
    pub fn new(url: String) -> Endpoint {
        Self {
            path: url,
            query: Default::default(),
            method: Default::default(),
            proxy: Default::default(),
            input: Default::default(),
            output: Default::default(),
            headers: Default::default(),
            body: Default::default(),
            description: Default::default(),
            encoding: Default::default(),
        }
    }
}
