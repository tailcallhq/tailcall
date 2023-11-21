#![allow(clippy::too_many_arguments)]

use derive_setters::Setters;
use hyper::HeaderMap;
use reqwest::header::{HeaderName, HeaderValue};

use crate::config::{GraphQLOperationType, KeyValues};
use crate::has_headers::HasHeaders;
use crate::http::Method::POST;
use crate::mustache::Mustache;
use crate::path_string::PathString;

/// RequestTemplate for GraphQL requests (See RequestTemplate
/// documentation)
#[derive(Setters, Debug, Clone)]
pub struct GraphqlRequestTemplate {
  pub url: String,
  pub operation_type: GraphQLOperationType,
  pub operation_name: String,
  pub operation_arguments: Option<String>,
  pub variable_definitions: Option<String>,
  pub variable_values: Vec<(String, Mustache)>,
  pub selection_set: Mustache,
  pub headers: Vec<(HeaderName, Mustache)>,
}

impl GraphqlRequestTemplate {
  fn create_headers<C: PathString>(&self, ctx: &C) -> HeaderMap {
    let mut header_map = HeaderMap::new();

    for (k, v) in &self.headers {
      if let Ok(header_value) = HeaderValue::from_str(&v.render(ctx)) {
        header_map.insert(k, header_value);
      }
    }

    header_map
  }

  fn set_headers<C: PathString + HasHeaders>(&self, mut req: reqwest::Request, ctx: &C) -> reqwest::Request {
    let headers = req.headers_mut();
    let config_headers = self.create_headers(ctx);

    if !config_headers.is_empty() {
      headers.extend(config_headers);
    }
    headers.insert(
      reqwest::header::CONTENT_TYPE,
      HeaderValue::from_static("application/json"),
    );
    headers.extend(ctx.headers().to_owned());
    req
  }

  pub fn to_request<C: PathString + HasHeaders>(&self, ctx: &C) -> anyhow::Result<reqwest::Request> {
    let mut req = reqwest::Request::new(POST.to_hyper(), url::Url::parse(self.url.as_str())?);
    req = self.set_headers(req, ctx);
    req = self.set_body(req, ctx);
    Ok(req)
  }

  fn set_body<C: PathString + HasHeaders>(&self, mut req: reqwest::Request, ctx: &C) -> reqwest::Request {
    let variable_values = self
      .variable_values
      .iter()
      .map(|(k, v)| (k, v.render(ctx)))
      // TODO: proper conversion from mustache to JSON value that will put quotes only when needed for JSON
      .map(|(k, v)| format!(r#""{}": "{}""#, k, v))
      .collect::<Vec<_>>()
      .join(",");
    let selection_set = self.selection_set.render(ctx);
    let operation = self
      .variable_definitions
      .as_ref()
      .map(|defs| format!("{}({})", self.operation_type, defs))
      .unwrap_or(self.operation_type.to_string());
    let query_name = self
      .operation_arguments
      .as_ref()
      .map(|args| format!("{}({})", self.operation_name, args))
      .unwrap_or(self.operation_name.clone());
    let graphql_query = format!(
      r#"{{ "query": "{operation} {{ {query_name} {{ {selection_set} }} }}", "variables": {{ {variable_values} }} }}"#,
    );

    req.body_mut().replace(graphql_query.into());
    req
  }

  pub fn new(
    url: String,
    operation_type: &GraphQLOperationType,
    operation_name: &str,
    args: Option<&KeyValues>,
    variable_definitions: Option<String>,
    headers: HeaderMap<HeaderValue>,
  ) -> anyhow::Result<Self> {
    let mut variable_values = Vec::new();
    let mut query_arguments = None;

    if let Some(args) = args.as_ref() {
      variable_values = args
        .iter()
        .map(|(k, v)| Ok((k.to_owned(), Mustache::parse(v)?)))
        .collect::<anyhow::Result<Vec<_>>>()?;
      query_arguments = Some(
        args
          .iter()
          .map(|(k, _)| format!("{}: ${}", k, k))
          .collect::<Vec<_>>()
          .join(","),
      );
    }

    let selection_set = Mustache::parse("{{field.selectionSet}}")?;
    let headers = headers
      .iter()
      .map(|(k, v)| Ok((k.clone(), Mustache::parse(v.to_str()?)?)))
      .collect::<anyhow::Result<Vec<_>>>()?;

    Ok(Self {
      url,
      operation_type: operation_type.to_owned(),
      operation_name: operation_name.to_owned(),
      operation_arguments: query_arguments,
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

  use crate::config::GraphQLOperationType;
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
  fn test_query_without_args() {
    let tmpl = GraphqlRequestTemplate::new(
      "http://localhost:3000".to_string(),
      &GraphQLOperationType::Query,
      "myQuery",
      None,
      None,
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
      r#"{ "query": "query { myQuery { a,b,c } }", "variables": {  } }"#
    );
  }

  #[test]
  fn test_query_with_args() {
    let tmpl = GraphqlRequestTemplate::new(
      "http://localhost:3000".to_string(),
      &GraphQLOperationType::Mutation,
      "create",
      Some(serde_json::from_str(r#"[{"key": "id", "value": "{{foo.bar}}"}]"#).unwrap()).as_ref(),
      Some("$id: Int".to_string()),
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
      r#"{ "query": "mutation($id: Int) { create(id: $id) { a,b,c } }", "variables": { "id": "baz" } }"#
    );
  }
}
