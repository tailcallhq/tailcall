use derive_setters::Setters;
use hyper::HeaderMap;
use reqwest::header::{HeaderName, HeaderValue};
use url::Url;

use crate::endpoint::Endpoint;
use crate::has_headers::HasHeaders;
use crate::mustache::Mustache;
use crate::path_string::PathString;

/// A template to quickly create a request
#[derive(Setters, Clone, Debug)]
pub struct RequestTemplate {
  pub root_url: Mustache,
  pub query: Vec<(String, Mustache)>,
  pub method: reqwest::Method,
  pub headers: Vec<(String, Mustache)>,
  pub body: Option<Mustache>,
  pub endpoint: Endpoint,
}

impl RequestTemplate {
  fn eval_url<C: PathString>(&self, ctx: &C) -> anyhow::Result<Url> {
    let root_url = self.root_url.render(ctx);
    let mut url = url::Url::parse(root_url.as_str())?;
    if !self.query.is_empty() {
      let query = self
        .query
        .iter()
        .map(|(k, v)| (k.as_str(), v.render(ctx)))
        .collect::<Vec<_>>();
      url.set_query(Some(&serde_urlencoded::to_string(query)?));
    }
    Ok(url)
  }
  fn eval_headers<C: PathString>(&self, ctx: &C) -> HeaderMap {
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

  fn eval_body<C: PathString>(&self, ctx: &C) -> reqwest::Body {
    self
      .body
      .as_ref()
      .map(|b| b.render(ctx).into())
      .unwrap_or(reqwest::Body::from("".to_string()))
  }

  /// A high-performance way to reliably create a request
  pub fn to_request<C: PathString + HasHeaders>(&self, ctx: &C) -> anyhow::Result<reqwest::Request> {
    let url = self.eval_url(ctx)?;
    let mut header_map = self.eval_headers(ctx);
    header_map.extend(ctx.headers().to_owned());
    header_map.insert(
      reqwest::header::CONTENT_TYPE,
      HeaderValue::from_static("application/json"),
    );
    let body = self.eval_body(ctx);
    let method = self.method.clone();
    let mut req = reqwest::Request::new(method, url);
    req.headers_mut().extend(header_map);
    req.body_mut().replace(body);
    Ok(req)
  }

  pub fn new(root_url: &str) -> anyhow::Result<Self> {
    Ok(Self {
      root_url: Mustache::parse(root_url)?,
      query: Default::default(),
      method: reqwest::Method::GET,
      headers: Default::default(),
      body: Default::default(),
      endpoint: Endpoint::new(root_url.to_string()),
    })
  }
}

impl TryFrom<Endpoint> for RequestTemplate {
  type Error = anyhow::Error;
  fn try_from(endpoint: Endpoint) -> anyhow::Result<Self> {
    let path = Mustache::parse(endpoint.path.as_str())?;
    let query = endpoint
      .query
      .iter()
      .map(|(k, v)| Ok((k.to_owned(), Mustache::parse(v.as_str())?)))
      .collect::<anyhow::Result<Vec<_>>>()?;
    let method = endpoint.method.clone().into();
    let headers = endpoint
      .headers
      .iter()
      .map(|(k, v)| Ok((k.as_str().into(), Mustache::parse(v.to_str()?)?)))
      .collect::<anyhow::Result<Vec<_>>>()?;

    let body = if let Some(body) = &endpoint.body {
      Some(Mustache::parse(body.as_str())?)
    } else {
      None
    };

    Ok(Self { root_url: path, query, method, headers, body, endpoint })
  }
}

#[cfg(test)]
mod tests {
  use std::borrow::Cow;

  use hyper::HeaderMap;
  use pretty_assertions::assert_eq;
  use serde_json::json;

  use crate::mustache::Mustache;
  use crate::request_template::RequestTemplate;
  struct TestContext {
    pub value: serde_json::Value,
    pub headers: HeaderMap,
  }
  impl TestContext {
    pub fn new(value: serde_json::Value, headers: HeaderMap) -> Self {
      Self { value, headers }
    }
  }
  impl crate::path_string::PathString for TestContext {
    fn path_string(&self, parts: &[String]) -> Option<Cow<'_, str>> {
      self.value.path_string(parts)
    }
  }
  impl crate::has_headers::HasHeaders for TestContext {
    fn headers(&self) -> &HeaderMap {
      &self.headers
    }
  }
  #[test]
  fn test_url() {
    let tmpl = RequestTemplate::new("http://localhost:3000/").unwrap();
    let ctx = TestContext { value: serde_json::Value::Null, headers: HeaderMap::new() };
    let req = tmpl.to_request(&ctx).unwrap();
    assert_eq!(req.url().to_string(), "http://localhost:3000/");
  }

  #[test]
  fn test_url_path() {
    let tmpl = RequestTemplate::new("http://localhost:3000/foo/bar").unwrap();
    let ctx = TestContext { value: serde_json::Value::Null, headers: HeaderMap::new() };
    let req = tmpl.to_request(&ctx).unwrap();
    assert_eq!(req.url().to_string(), "http://localhost:3000/foo/bar");
  }

  #[test]
  fn test_url_path_template() {
    let tmpl = RequestTemplate::new("http://localhost:3000/foo/{{bar.baz}}").unwrap();
    let ctx = TestContext::new(
      json!({
        "bar": {
          "baz": "bar"
        }
      }),
      HeaderMap::new(),
    );

    let req = tmpl.to_request(&ctx).unwrap();
    assert_eq!(req.url().to_string(), "http://localhost:3000/foo/bar");
  }
  #[test]
  fn test_url_path_template_multi() {
    let tmpl = RequestTemplate::new("http://localhost:3000/foo/{{bar.baz}}/boozes/{{bar.booz}}").unwrap();
    let ctx = TestContext::new(
      json!({
        "bar": {
          "baz": "bar",
          "booz": 1
        }
      }),
      HeaderMap::new(),
    );
    let req = tmpl.to_request(&ctx).unwrap();
    assert_eq!(req.url().to_string(), "http://localhost:3000/foo/bar/boozes/1");
  }
  #[test]
  fn test_url_query_params() {
    let query = vec![
      ("foo".to_string(), Mustache::parse("0").unwrap()),
      ("bar".to_string(), Mustache::parse("1").unwrap()),
      ("baz".to_string(), Mustache::parse("2").unwrap()),
    ];
    let tmpl = RequestTemplate::new("http://localhost:3000").unwrap().query(query);
    let ctx = TestContext::new(serde_json::Value::Null, HeaderMap::new());
    let req = tmpl.to_request(&ctx).unwrap();
    assert_eq!(req.url().to_string(), "http://localhost:3000/?foo=0&bar=1&baz=2");
  }
  #[test]
  fn test_url_query_params_template() {
    let query = vec![
      ("foo".to_string(), Mustache::parse("0").unwrap()),
      ("bar".to_string(), Mustache::parse("{{bar.id}}").unwrap()),
      ("baz".to_string(), Mustache::parse("{{baz.id}}").unwrap()),
    ];
    let tmpl = RequestTemplate::new("http://localhost:3000/").unwrap().query(query);
    let ctx = TestContext::new(
      json!({
        "bar": {
          "id": 1
        },
        "baz": {
          "id": 2
        }
      }),
      HeaderMap::new(),
    );
    let req = tmpl.to_request(&ctx).unwrap();
    assert_eq!(req.url().to_string(), "http://localhost:3000/?foo=0&bar=1&baz=2");
  }
  #[test]
  fn test_headers() {
    let headers = vec![
      ("foo".to_string(), Mustache::parse("foo").unwrap()),
      ("bar".to_string(), Mustache::parse("bar").unwrap()),
      ("baz".to_string(), Mustache::parse("baz").unwrap()),
    ];
    let tmpl = RequestTemplate::new("http://localhost:3000").unwrap().headers(headers);
    let ctx = TestContext::new(serde_json::Value::Null, HeaderMap::new());
    let req = tmpl.to_request(&ctx).unwrap();
    assert_eq!(req.headers().get("foo").unwrap(), "foo");
    assert_eq!(req.headers().get("bar").unwrap(), "bar");
    assert_eq!(req.headers().get("baz").unwrap(), "baz");
  }
  #[test]
  fn test_header_template() {
    let headers = vec![
      ("foo".to_string(), Mustache::parse("0").unwrap()),
      ("bar".to_string(), Mustache::parse("{{bar.id}}").unwrap()),
      ("baz".to_string(), Mustache::parse("{{baz.id}}").unwrap()),
    ];
    let tmpl = RequestTemplate::new("http://localhost:3000").unwrap().headers(headers);
    let ctx = TestContext::new(
      json!({
        "bar": {
          "id": 1
        },
        "baz": {
          "id": 2
        }
      }),
      HeaderMap::new(),
    );
    let req = tmpl.to_request(&ctx).unwrap();
    assert_eq!(req.headers().get("foo").unwrap(), "0");
    assert_eq!(req.headers().get("bar").unwrap(), "1");
    assert_eq!(req.headers().get("baz").unwrap(), "2");
  }
  #[test]
  fn test_method() {
    let tmpl = RequestTemplate::new("http://localhost:3000")
      .unwrap()
      .method(reqwest::Method::POST);
    let ctx = TestContext::new(serde_json::Value::Null, HeaderMap::new());
    let req = tmpl.to_request(&ctx).unwrap();
    assert_eq!(req.method(), reqwest::Method::POST);
  }
  #[test]
  fn test_body() {
    let tmpl = RequestTemplate::new("http://localhost:3000")
      .unwrap()
      .body(Some(Mustache::parse("foo").unwrap()));
    let ctx = TestContext::new(serde_json::Value::Null, HeaderMap::new());
    let body = tmpl
      .to_request(&ctx)
      .unwrap()
      .body()
      .unwrap()
      .as_bytes()
      .unwrap()
      .to_owned();
    assert_eq!(body, "foo".as_bytes());
  }
  #[test]
  fn test_body_template() {
    let tmpl = RequestTemplate::new("http://localhost:3000")
      .unwrap()
      .body(Some(Mustache::parse("{{foo.bar}}").unwrap()));
    let ctx = TestContext::new(
      json!({
        "foo": {
          "bar": "baz"
        }
      }),
      HeaderMap::new(),
    );
    let body = tmpl
      .to_request(&ctx)
      .unwrap()
      .body()
      .unwrap()
      .as_bytes()
      .unwrap()
      .to_owned();
    assert_eq!(body, "baz".as_bytes());
  }
  #[test]
  fn test_from_endpoint() {
    let mut headers = HeaderMap::new();
    headers.insert("foo", "bar".parse().unwrap());
    let endpoint = crate::endpoint::Endpoint::new("http://localhost:3000/".to_string())
      .method(crate::http::Method::POST)
      .headers(headers)
      .body(Some("foo".into()));
    let tmpl = RequestTemplate::try_from(endpoint).unwrap();
    let ctx = TestContext::new(serde_json::Value::Null, HeaderMap::new());
    let req = tmpl.to_request(&ctx).unwrap();
    assert_eq!(req.method(), reqwest::Method::POST);
    assert_eq!(req.headers().get("foo").unwrap(), "bar");
    let body = req.body().unwrap().as_bytes().unwrap().to_owned();
    assert_eq!(body, "foo".as_bytes());
    assert_eq!(req.url().to_string(), "http://localhost:3000/");
  }
  #[test]
  fn test_from_endpoint_template() {
    let mut headers = HeaderMap::new();
    headers.insert("foo", "{{foo.header}}".parse().unwrap());
    let endpoint = crate::endpoint::Endpoint::new("http://localhost:3000/{{foo.bar}}".to_string())
      .method(crate::http::Method::POST)
      .query(vec![("foo".to_string(), "{{foo.bar}}".to_string())])
      .headers(headers)
      .body(Some("{{foo.bar}}".into()));
    let tmpl = RequestTemplate::try_from(endpoint).unwrap();
    let ctx = TestContext::new(
      json!({
        "foo": {
          "bar": "baz",
          "header": "abc"
        }
      }),
      HeaderMap::new(),
    );
    let req = tmpl.to_request(&ctx).unwrap();
    assert_eq!(req.method(), reqwest::Method::POST);
    assert_eq!(req.headers().get("foo").unwrap(), "abc");
    let body = req.body().unwrap().as_bytes().unwrap().to_owned();
    assert_eq!(body, "baz".as_bytes());
    assert_eq!(req.url().to_string(), "http://localhost:3000/baz?foo=baz");
  }
  #[test]
  fn test_headers_forward() {
    let endpoint = crate::endpoint::Endpoint::new("http://localhost:3000/".to_string());
    let tmpl = RequestTemplate::try_from(endpoint).unwrap();
    let mut headers = HeaderMap::new();
    headers.insert("baz", "qux".parse().unwrap());
    let ctx = TestContext::new(serde_json::Value::Null, headers);
    let req = tmpl.to_request(&ctx).unwrap();
    assert_eq!(req.headers().get("baz").unwrap(), "qux");
  }
}
