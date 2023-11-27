// src/custom_common.rs

use derive_setters::Setters;
use hyper::HeaderMap;

use crate::endpoint::Endpoint;
use crate::has_headers::HasHeaders;
use crate::path_string::PathString;
use crate::request_template::RequestTemplate;

#[derive(Setters)]
pub struct Context {
  pub value: serde_json::Value,
  pub headers: HeaderMap,
}

impl Default for Context {
  fn default() -> Self {
    Self { value: serde_json::Value::Null, headers: HeaderMap::new() }
  }
}

impl PathString for Context {
  fn path_string<T: AsRef<str>>(&self, parts: &[T]) -> Option<std::borrow::Cow<'_, str>> {
    self.value.path_string(parts)
  }
}

impl HasHeaders for Context {
  fn headers(&self) -> &HeaderMap {
    &self.headers
  }
}

pub fn create_request_templates() -> (RequestTemplate, RequestTemplate) {
  let tmpl_mustache = RequestTemplate::try_from(Endpoint::new(
    "http://localhost:3000/{{args.b}}?a={{args.a}}&b={{args.b}}&c={{args.c}}".to_string(),
  ))
  .unwrap();

  let tmpl_literal =
    RequestTemplate::try_from(Endpoint::new("http://localhost:3000/foo?a=bar&b=foo&c=baz".to_string())).unwrap();

  (tmpl_literal, tmpl_mustache)
}
