#![allow(clippy::too_many_arguments)]

use std::collections::BTreeMap;
use std::sync::Arc;

use anyhow::Result;
use derive_setters::Setters;
use hyper::http::{HeaderName, HeaderValue};
use hyper::HeaderMap;
use serde_json::json;
use url::Url;

use crate::batch::Batch;
use crate::http::{Method, Scheme};
use crate::inet_address::InetAddress;
use crate::json::{JsonLike, JsonSchema};
use crate::mustache::Mustache;
use crate::path::{Path, Segment};

const EMPTY_VEC: &Vec<String> = &vec![];

#[derive(Clone, Debug, Setters)]
pub struct Endpoint {
  pub method: Method,
  pub path: Path,
  pub query: Vec<(String, Mustache)>,
  pub address: InetAddress,
  pub input: JsonSchema,
  pub output: JsonSchema,
  pub headers: Vec<(String, String)>,
  pub scheme: Scheme,
  pub body: Option<Mustache>,
  pub description: Option<String>,
  pub batch: Option<Batch>,
  pub list: Option<bool>,

  // TODO: endpoint can be compiled to request in static cases
  pub request: Option<Arc<reqwest::Request>>,
}

impl Endpoint {
  pub fn new(address: InetAddress) -> Endpoint {
    Endpoint {
      method: Method::GET,
      path: Path::default(),
      query: Vec::new(),
      address,
      input: JsonSchema::default(),
      output: JsonSchema::default(),
      headers: vec![],
      scheme: Scheme::Http,
      body: None,
      description: None,
      batch: None,
      list: None,
      request: None,
    }
  }

  pub fn batch_key(&self) -> Option<&String> {
    match self.batch {
      None => None,
      Some(ref batch) => Some(batch.key()),
    }
  }

  pub fn batch_path(&self) -> &Vec<String> {
    match self.batch {
      None => EMPTY_VEC,
      Some(ref batch) => batch.path(),
    }
  }

  pub fn port(mut self, port: u16) -> Endpoint {
    assert!(port > 0 && port < 65535);
    self.address = self.address.port(port);
    self
  }

  pub fn is_batched(&self) -> bool {
    self.batch.is_some()
  }

  pub fn to_request(
    &self,
    input: &async_graphql::Value,
    env: Option<&async_graphql::Value>,
    args: Option<&async_graphql::Value>,
    headers: &HeaderMap,
  ) -> Result<reqwest::Request> {
    let url = self.get_url(input, env, args, headers)?;
    let method: reqwest::Method = self.method.clone().into();
    let mut request = reqwest::Request::new(method, url);
    let headers = self.eval_headers(input, env, args, headers)?;
    let body = self.body_str(input, env, args, &headers);
    request.headers_mut().extend(headers);
    request.headers_mut().insert(
      reqwest::header::CONTENT_TYPE,
      reqwest::header::HeaderValue::from_static("application/json"),
    );
    request.body_mut().replace(reqwest::Body::from(body));
    Ok(request)
  }

  const VALUE_STR: &'static str = "value";
  const VARS_STR: &'static str = "vars";
  const ARGS_STR: &'static str = "args";
  const HEADERS_STR: &'static str = "headers";
  fn get_header(key: &str, headers: &HeaderMap) -> Option<async_graphql::Value> {
    headers.get(key).and_then(|value| {
      value
        .to_str()
        .ok()
        .and_then(|str_val| async_graphql::Value::from_json(json!(str_val)).ok())
    })
  }
  fn extract_value_from_path(
    &self,
    part: &str,
    parts: &[String],
    input: &async_graphql::Value,
    env: &async_graphql::Value,
    args: &async_graphql::Value,
    headers: &HeaderMap,
  ) -> Option<String> {
    let header;
    match part {
      Self::VALUE_STR => input.get_path(&parts[1..]),
      Self::VARS_STR => env.get_path(&parts[1..]),
      Self::ARGS_STR => args.get_path(&parts[1..]),
      Self::HEADERS_STR => {
        header = Self::get_header(&parts[1], headers);
        header.as_ref()
      }
      _ => None,
    }
    .and_then(|value| match value {
      async_graphql::Value::String(str_val) => Some(str_val.clone()),
      async_graphql::Value::Number(num_val) => Some(num_val.to_string()),
      async_graphql::Value::Boolean(bool_val) => Some(bool_val.to_string()),
      async_graphql::Value::Object(map) => Some(json!(map).to_string()),
      async_graphql::Value::List(list) => Some(json!(list).to_string()),
      _ => None,
    })
  }

  fn eval_headers(
    &self,
    input: &async_graphql::Value,
    env: Option<&async_graphql::Value>,
    args: Option<&async_graphql::Value>,
    headers: &HeaderMap,
  ) -> Result<HeaderMap> {
    let mut finalheaders: HeaderMap = HeaderMap::new();
    finalheaders.extend(headers.to_owned());
    let env = env.unwrap_or(&async_graphql::Value::Null);
    let args = args.unwrap_or(&async_graphql::Value::Null);
    if !self.headers.is_empty() {
      for (key, value) in &self.query {
        match value {
          Mustache::Simple(s) => {
            finalheaders.insert(
              key.clone().as_str().parse::<HeaderName>()?,
              s.clone().parse::<HeaderValue>()?,
            );
          }
          Mustache::Template(parts) => {
            if let Some(part) = parts.first() {
              if let Some(result) =
                self.extract_value_from_path(part, parts, input, env, args, &finalheaders.to_owned())
              {
                finalheaders.insert(
                  key.clone().as_str().parse::<HeaderName>()?,
                  result.as_str().parse::<HeaderValue>()?,
                );
              }
            }
          }
        }
      }
    }
    Ok(finalheaders)
  }

  pub fn body_str(
    &self,
    input: &async_graphql::Value,
    env: Option<&async_graphql::Value>,
    args: Option<&async_graphql::Value>,
    headers: &HeaderMap,
  ) -> String {
    let body = self.body.as_ref();
    let env = env.unwrap_or(&async_graphql::Value::Null);
    let args = args.unwrap_or(&async_graphql::Value::Null);
    let mut s = String::new();

    if let Some(body) = body {
      match body {
        Mustache::Simple(str) => s.push_str(str),
        Mustache::Template(parts) => {
          if let Some(part) = parts.first() {
            if let Some(result) = self.extract_value_from_path(part, parts, input, env, args, headers) {
              s.push_str(&result);
            }
          }
        }
      }
    }
    s
  }

  pub fn get_url(
    &self,
    input: &async_graphql::Value,
    env: Option<&async_graphql::Value>,
    args: Option<&async_graphql::Value>,
    headers: &HeaderMap,
  ) -> Result<Url> {
    let mut url = Url::parse(&format!("{}://{}", self.scheme, self.address))?;

    let env = env.unwrap_or(&async_graphql::Value::Null);
    let args = args.unwrap_or(&async_graphql::Value::Null);

    url.set_path(&self.eval_path(input, env, args, headers));

    if !self.query.is_empty() {
      let mut query_params = BTreeMap::new();
      for (key, value) in &self.query {
        match value {
          Mustache::Simple(s) => {
            query_params.insert(key.clone(), s.clone());
          }
          Mustache::Template(parts) => {
            if let Some(part) = parts.first() {
              if let Some(result) = self.extract_value_from_path(part, parts, input, env, args, headers) {
                query_params.insert(key.clone(), result);
              }
            }
          }
        }
      }
      url.set_query(Some(&serde_urlencoded::to_string(&query_params)?));
    }
    Ok(url)
  }

  pub fn eval_path(
    &self,
    input: &async_graphql::Value,
    env: &async_graphql::Value,
    args: &async_graphql::Value,
    headers: &HeaderMap,
  ) -> String {
    let mut s = String::new();
    for (i, segment) in self.path.segments.iter().enumerate() {
      if i > 0 {
        s.push('/');
      }
      match segment {
        Segment::Literal { value } => s.push_str(value),
        Segment::Param { location } => {
          let parts = location;
          if let Some(part) = parts.first() {
            if let Some(result) = self.extract_value_from_path(part, parts, input, env, args, headers) {
              s.push_str(&result);
            }
          }
        }
      }
    }
    s
  }
}
#[cfg(test)]
mod tests {
  use hyper::header::{HeaderName, HeaderValue};
  use hyper::HeaderMap;
  use serde_json::json;

  use crate::endpoint::Endpoint;
  use crate::inet_address::InetAddress;
  use crate::mustache::Mustache;
  use crate::path::{Path, Segment};

  #[test]
  fn test_get_url() {
    let endpoint = Endpoint::new(InetAddress::new("localhost".to_string(), 8080)).path(Path::new(vec![
      Segment::literal("api".to_string()),
      Segment::literal("v1".to_string()),
      Segment::literal("users".to_string()),
    ]));
    let result = endpoint
      .get_url(&async_graphql::Value::Null, None, None, &HeaderMap::new())
      .unwrap()
      .to_string();
    assert_eq!(result, "http://localhost:8080/api/v1/users");
  }
  #[test]
  fn test_get_url_with_param() {
    let endpoint = Endpoint::new(InetAddress::new("localhost".to_string(), 8080))
      .path(Path::new(vec![
        Segment::literal("api".to_string()),
        Segment::literal("v1".to_string()),
      ]))
      .query(
        [
          ("a".to_string(), Mustache::Simple("1".to_string())),
          ("b".to_string(), Mustache::Simple("2".to_string())),
          ("c".to_string(), Mustache::Simple("3".to_string())),
        ]
        .to_vec(),
      );
    let result = endpoint
      .get_url(&async_graphql::Value::Null, None, None, &HeaderMap::new())
      .unwrap()
      .to_string();
    assert_eq!(result, "http://localhost:8080/api/v1?a=1&b=2&c=3");
  }
  #[test]
  fn test_get_url_with_param_mustache() {
    let endpoint = Endpoint::new(InetAddress::new("localhost".to_string(), 8080))
      .path(Path::new(vec![
        Segment::literal("api".to_string()),
        Segment::literal("v1".to_string()),
      ]))
      .query(
        [
          ("a".to_string(), Mustache::Simple("1".to_string())),
          ("b".to_string(), Mustache::Simple("2".to_string())),
          (
            "c".to_string(),
            Mustache::Template(vec!["vars".to_string(), "name".to_string()]),
          ),
        ]
        .to_vec(),
      );
    let result = endpoint
      .get_url(
        &async_graphql::Value::Null,
        async_graphql::Value::from_json(json!( {"name": 3})).ok().as_ref(),
        None,
        &HeaderMap::new(),
      )
      .unwrap()
      .to_string();
    assert_eq!(result, "http://localhost:8080/api/v1?a=1&b=2&c=3");
  }
  #[test]
  fn test_get_url_with_url_param() {
    let endpoint = Endpoint::new(InetAddress::new("localhost".to_string(), 8080)).path(Path::new(vec![
      Segment::literal("api".to_string()),
      Segment::literal("v1".to_string()),
      Segment::param(vec!["value".to_string(), "id".to_string()]),
    ]));
    let result = endpoint
      .get_url(
        &async_graphql::Value::from_json(json!({"id": 123})).unwrap(),
        None,
        None,
        &HeaderMap::new(),
      )
      .unwrap()
      .to_string();
    assert_eq!(result, "http://localhost:8080/api/v1/123");
  }
  #[test]
  fn test_get_url_with_url_param_args() {
    let endpoint = Endpoint::new(InetAddress::new("localhost".to_string(), 8080)).path(Path::new(vec![
      Segment::literal("api".to_string()),
      Segment::literal("v1".to_string()),
      Segment::param(vec!["args".to_string(), "id".to_string()]),
    ]));
    let result = endpoint
      .get_url(
        &async_graphql::Value::Null,
        None,
        Some(&async_graphql::Value::from_json(json!({"id": 123})).unwrap()),
        &HeaderMap::new(),
      )
      .unwrap()
      .to_string();
    assert_eq!(result, "http://localhost:8080/api/v1/123");
  }
  #[test]
  fn test_get_url_with_url_param_headers() {
    let endpoint = Endpoint::new(InetAddress::new("localhost".to_string(), 8080)).path(Path::new(vec![
      Segment::literal("api".to_string()),
      Segment::literal("v1".to_string()),
      Segment::param(vec!["headers".to_string(), "id".to_string()]),
    ]));
    let headers = {
      let mut headers = HeaderMap::new();
      headers.insert(HeaderName::from_static("id"), HeaderValue::from_static("123"));
      headers
    };
    let result = endpoint
      .get_url(&async_graphql::Value::Null, None, None, &headers)
      .unwrap()
      .to_string();
    assert_eq!(result, "http://localhost:8080/api/v1/123");
  }
}
