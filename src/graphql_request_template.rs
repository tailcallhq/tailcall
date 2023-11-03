#![allow(clippy::too_many_arguments)]

use derive_setters::Setters;
use hyper::HeaderMap;
use reqwest::header::{HeaderName, HeaderValue};

use crate::has_headers::HasHeaders;
use crate::http::Method::POST;
use crate::mustache::Mustache;
use crate::path_string::PathString;

/// RequestTemplate for GraphQL requests (See RequestTemplate
/// documentation)
#[derive(Setters, Debug, Clone)]
pub struct GraphqlRequestTemplate {
  pub url: String,
  pub query_name: String,
  pub query_arguments: String,
  pub variable_definitions: String,
  pub variable_values: Vec<(String, Mustache)>,
  pub selection_set: Mustache,
  pub headers: Vec<(String, Mustache)>,
}

impl GraphqlRequestTemplate {
  fn create_headers<C: PathString>(&self, ctx: &C) -> HeaderMap {
    let mut header_map = HeaderMap::new();

    for (k, v) in &self.headers {
      if let Ok(header_name) = HeaderName::from_bytes(k.as_bytes()) {
        if let Ok(header_value) = HeaderValue::from_str(&v.render(ctx)) {
          header_map.insert(header_name, header_value);
        }
      }
    }

    header_map
  }

  fn set_headers<C: PathString + HasHeaders>(&self, mut req: reqwest::Request, ctx: &C) -> reqwest::Request {
    let headers = self.create_headers(ctx);
    if !headers.is_empty() {
      req.headers_mut().extend(headers);
    }

    let headers = req.headers_mut();
    headers.insert(
      reqwest::header::CONTENT_TYPE,
      HeaderValue::from_static("application/json"),
    );
    headers.extend(ctx.headers().to_owned());
    req
  }

  pub fn to_request<C: PathString + HasHeaders>(&self, ctx: &C) -> anyhow::Result<reqwest::Request> {
    let mut req = reqwest::Request::new(POST.into(), url::Url::parse(self.url.as_str())?);
    req = self.set_headers(req, ctx);
    req = self.set_body(req, ctx);
    Ok(req)
  }

  fn set_body<C: PathString + HasHeaders>(&self, mut req: reqwest::Request, ctx: &C) -> reqwest::Request {
    let variable_values = self
      .variable_values
      .iter()
      .map(|(k, v)| (k, v.render(ctx)))
      .map(|(k, v)| format!(r#""{}": {}"#, k, v))
      .collect::<Vec<_>>()
      .join(",");
    let selection_set = self.selection_set.render(ctx);
    let graphql_query = format!(
      r#"{{ "query": "query({}) {{ {}({}) {{ {} }} }}", "variables": {{ {} }} }}"#,
      self.variable_definitions, self.query_name, self.query_arguments, selection_set, variable_values
    );
    req.body_mut().replace(graphql_query.into());
    req
  }

  pub fn new(
    url: String,
    query_name: String,
    args: Vec<(String, String)>,
    variable_definitions: String,
    headers: HeaderMap<HeaderValue>,
  ) -> anyhow::Result<Self> {
    let variable_values = args
      .clone()
      .iter()
      .map(|(k, v)| Ok((k.to_owned(), Mustache::parse(v.as_str())?)))
      .collect::<anyhow::Result<Vec<_>>>()?;
    let arguments = args
      .clone()
      .iter()
      .map(|(k, _)| format!("{}: ${}", k, k))
      .collect::<Vec<_>>()
      .join(",");
    let selection_set = Mustache::parse("{{field.selectionSet}}")?;
    let headers = headers
      .iter()
      .map(|(k, v)| Ok((k.as_str().into(), Mustache::parse(v.to_str()?)?)))
      .collect::<anyhow::Result<Vec<_>>>()?;
    Ok(Self {
      url,
      query_name,
      query_arguments: arguments,
      variable_definitions,
      variable_values,
      selection_set,
      headers,
    })
  }
}

#[cfg(test)]
mod tests {
  use std::borrow::Cow;

  use derive_setters::Setters;
  use hyper::HeaderMap;
  use pretty_assertions::assert_eq;
  use serde_json::json;

  use crate::graphql_request_template::GraphqlRequestTemplate;

  #[derive(Setters)]
  struct Context {
    pub value: serde_json::Value,
    pub headers: HeaderMap,
  }

  impl Default for Context {
    fn default() -> Self {
      Self { value: serde_json::Value::Null, headers: HeaderMap::new() }
    }
  }
  impl crate::path_string::PathString for Context {
    fn path_string<T: AsRef<str>>(&self, parts: &[T]) -> Option<Cow<'_, str>> {
      self.value.path_string(parts)
    }
  }
  impl crate::has_headers::HasHeaders for Context {
    fn headers(&self) -> &HeaderMap {
      &self.headers
    }
  }

  #[test]
  fn test_graphql_query() {
    let tmpl = GraphqlRequestTemplate::new(
      "http://localhost:3000".to_string(),
      "myQuery".to_string(),
      vec![("id".to_string(), "{{foo.bar}}".to_string())],
      "$id: Int".to_string(),
      HeaderMap::new(),
    )
    .unwrap();
    let ctx = Context::default().value(json!({
      "foo": {
        "bar": "baz",
        "header": "abc"
      },
      "field": {
        "selectionSet": "a,b,c"
      }
    }));

    let req = tmpl.to_request(&ctx).unwrap();
    let body = req.body().unwrap().as_bytes().unwrap().to_owned();

    assert_eq!(
      std::str::from_utf8(&body).unwrap(),
      r#"{ "query": "query($id: Int) { myQuery(id: $id) { a,b,c } }", "variables": { "id": baz } }"#
    );
  }
}
