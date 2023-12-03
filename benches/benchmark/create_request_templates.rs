use derive_setters::Setters;
use hyper::HeaderMap;
use tailcall::endpoint::Endpoint;
use tailcall::has_headers::HasHeaders;
use tailcall::path::PathString;
use tailcall::request_template::RequestTemplate;

// Context struct with setters using the derive_setters crate
#[derive(Setters)]
pub struct Context {
  pub value: serde_json::Value,
  pub headers: HeaderMap,
}

// default for Context, initializing value as Null and headers as an empty HeaderMap
impl Default for Context {
  fn default() -> Self {
    Self { value: serde_json::Value::Null, headers: HeaderMap::new() }
  }
}

// PathString for Context, delegating to the path_string method of the value field
impl PathString for Context {
  fn path_string<T: AsRef<str>>(&self, parts: &[T]) -> Option<std::borrow::Cow<'_, str>> {
    self.value.path_string(parts)
  }
}

// HasHeaders for Context, returning a reference to the headers field
impl HasHeaders for Context {
  fn headers(&self) -> &HeaderMap {
    &self.headers
  }
}

// Function to create request templates (literal and mustache)
#[allow(dead_code)]
pub fn create_request_templates() -> (RequestTemplate, RequestTemplate) {
  // Create a RequestTemplate using a literal endpoint
  let tmpl_mustache = RequestTemplate::try_from(Endpoint::new(
    "http://localhost:3000/{{args.b}}?a={{args.a}}&b={{args.b}}&c={{args.c}}".to_string(),
  ))
  .unwrap();

  // Create a RequestTemplate using a literal endpoint
  let tmpl_literal =
    RequestTemplate::try_from(Endpoint::new("http://localhost:3000/foo?a=bar&b=foo&c=baz".to_string())).unwrap();

  (tmpl_literal, tmpl_mustache)
}
