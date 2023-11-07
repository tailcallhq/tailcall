use derive_setters::Setters;
use hyper::HeaderMap;

use crate::http::Method;
use crate::json::JsonSchema;

#[derive(Clone, Debug, Setters)]
pub struct Endpoint {
  pub path: String,
  pub query: Vec<(String, String)>,
  pub method: Method,
  pub input: JsonSchema,
  pub output: JsonSchema,
  pub headers: HeaderMap,
  pub body: Option<String>,
  pub description: Option<String>,
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
    }
  }
}
