use std::borrow::Cow;

use derive_setters::Setters;
use hyper::HeaderMap;
use reqwest::header::{HeaderName, HeaderValue};
use serde_json::Value;
use url::Url;

use crate::endpoint_v2::Endpoint;
use crate::json::JsonLike;
use crate::mustache_v2::Mustache;

// TODO: move to it's own file
pub trait AnyPath {
  fn any_path(&self, path: &[String]) -> Option<Cow<'_, str>>;
}

impl AnyPath for serde_json::Value {
  fn any_path(&self, path: &[String]) -> Option<Cow<'_, str>> {
    self.get_path(path).and_then(|a| match a {
      Value::String(s) => Some(Cow::Borrowed(s.as_str())),
      Value::Number(n) => Some(Cow::Owned(n.to_string())),
      Value::Bool(b) => Some(Cow::Owned(b.to_string())),
      _ => None,
    })
  }
}

/// A template to quickly create a request
#[derive(Setters)]
pub struct RequestTemplate {
  pub root_url: Mustache,
  pub query: Vec<(String, Mustache)>,
  pub method: reqwest::Method,
  pub headers: Vec<(String, Mustache)>,
  pub body: Option<Mustache>,
}

impl RequestTemplate {
  fn eval_url<C: AnyPath>(&self, ctx: &C) -> Url {
    let root_url = self.root_url.render(ctx);
    let mut url = url::Url::parse(root_url.as_str()).unwrap();
    if !self.query.is_empty() {
      let query = self
        .query
        .iter()
        .map(|(k, v)| (k.as_str(), v.render(ctx)))
        .collect::<Vec<_>>();
      url.set_query(Some(&serde_urlencoded::to_string(query).unwrap()));
    }
    url
  }
  fn eval_headers<C: AnyPath>(&self, ctx: &C) -> HeaderMap {
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

  fn eval_body<C: AnyPath>(&self, ctx: &C) -> reqwest::Body {
    self
      .body
      .as_ref()
      .map(|b| b.render(ctx).into())
      .unwrap_or(reqwest::Body::from("".to_string()))
  }

  /// A high-performance way to reliably create a request
  pub fn to_request<C: AnyPath>(self, ctx: &C) -> reqwest::Request {
    let url = self.eval_url(ctx);
    let header_map = self.eval_headers(ctx);
    let body = self.eval_body(ctx);
    let mut req = reqwest::Request::new(self.method, url);
    req.headers_mut().extend(header_map);
    req.body_mut().replace(body);
    req
  }

  pub fn new(root_url: &str) -> anyhow::Result<Self> {
    Ok(Self {
      root_url: Mustache::parse(root_url)?,
      query: Default::default(),
      method: reqwest::Method::GET,
      headers: Default::default(),
      body: Default::default(),
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

    Ok(Self { root_url: path, query, method, headers, body })
  }
}

#[cfg(test)]
mod tests {
  use hyper::HeaderMap;
  use pretty_assertions::assert_eq;
  use serde_json::json;

  use crate::mustache_v2::Mustache;
  use crate::request_template::RequestTemplate;

  #[test]
  fn test_url() {
    let tmpl = RequestTemplate::new(r#""http://localhost:3000/""#).unwrap();
    let ctx = serde_json::Value::Null;
    let req = tmpl.to_request(&ctx);
    assert_eq!(req.url().to_string(), "http://localhost:3000/");
  }

  #[test]
  fn test_url_path() {
    let tmpl = RequestTemplate::new(r#""http://localhost:3000/foo/bar""#).unwrap();
    let ctx = serde_json::Value::Null;
    let req = tmpl.to_request(&ctx);
    assert_eq!(req.url().to_string(), "http://localhost:3000/foo/bar");
  }

  #[test]
  fn test_url_path_template() {
    let tmpl = RequestTemplate::new(r#""http://localhost:3000/foo/{{bar.baz}}""#).unwrap();
    let ctx = json!({
      "bar": {
        "baz": "bar"
      }
    });
    let req = tmpl.to_request(&ctx);
    assert_eq!(req.url().to_string(), "http://localhost:3000/foo/bar");
  }
  #[test]
  fn test_url_path_template_multi() {
    let tmpl = RequestTemplate::new(r#""http://localhost:3000/foo/{{bar.baz}}/boozes/{{bar.booz}}""#).unwrap();
    let ctx = json!({
      "bar": {
        "baz": "bar",
        "booz": 1
      }
    });
    let req = tmpl.to_request(&ctx);
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
    let ctx = serde_json::Value::Null;
    let req = tmpl.to_request(&ctx);
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
    let ctx = json!({
      "bar": {
        "id": 1
      },
      "baz": {
        "id": 2
      }
    });
    let req = tmpl.to_request(&ctx);
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
    let ctx = serde_json::Value::Null;
    let req = tmpl.to_request(&ctx);
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
    let ctx = json!({
      "bar": {
        "id": 1
      },
      "baz": {
        "id": 2
      }
    });
    let req = tmpl.to_request(&ctx);
    assert_eq!(req.headers().get("foo").unwrap(), "0");
    assert_eq!(req.headers().get("bar").unwrap(), "1");
    assert_eq!(req.headers().get("baz").unwrap(), "2");
  }
  #[test]
  fn test_method() {
    let tmpl = RequestTemplate::new("http://localhost:3000")
      .unwrap()
      .method(reqwest::Method::POST);
    let ctx = serde_json::Value::Null;
    let req = tmpl.to_request(&ctx);
    assert_eq!(req.method(), reqwest::Method::POST);
  }
  #[test]
  fn test_body() {
    let tmpl = RequestTemplate::new("http://localhost:3000")
      .unwrap()
      .body(Some(Mustache::parse("foo").unwrap()));
    let ctx = serde_json::Value::Null;
    let body = tmpl.to_request(&ctx).body().unwrap().as_bytes().unwrap().to_owned();
    assert_eq!(body, "foo".as_bytes());
  }
  #[test]
  fn test_body_template() {
    let tmpl = RequestTemplate::new("http://localhost:3000")
      .unwrap()
      .body(Some(Mustache::parse("{{foo.bar}}").unwrap()));
    let ctx = json!({
      "foo": {
        "bar": "baz"
      }
    });
    let body = tmpl.to_request(&ctx).body().unwrap().as_bytes().unwrap().to_owned();
    assert_eq!(body, "baz".as_bytes());
  }
  #[test]
  fn test_from_endpoint() {
    let mut headers = HeaderMap::new();
    headers.insert("foo", "bar".parse().unwrap());
    let endpoint = crate::endpoint_v2::Endpoint::new("http://localhost:3000/".to_string())
      .method(crate::http::Method::POST)
      .headers(headers)
      .body(Some("foo".into()));
    let tmpl = RequestTemplate::try_from(endpoint).unwrap();
    let ctx = serde_json::Value::Null;
    let req = tmpl.to_request(&ctx);
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
    let endpoint = crate::endpoint_v2::Endpoint::new("http://localhost:3000/{{foo.bar}}".to_string())
      .method(crate::http::Method::POST)
      .query(vec![("foo".to_string(), "{{foo.bar}}".to_string())])
      .headers(headers)
      .body(Some("{{foo.bar}}".into()));
    let tmpl = RequestTemplate::try_from(endpoint).unwrap();
    let ctx = json!({
      "foo": {
        "bar": "baz",
        "header": "abc"
      }
    });
    let req = tmpl.to_request(&ctx);
    assert_eq!(req.method(), reqwest::Method::POST);
    assert_eq!(req.headers().get("foo").unwrap(), "abc");
    let body = req.body().unwrap().as_bytes().unwrap().to_owned();
    assert_eq!(body, "baz".as_bytes());
    assert_eq!(req.url().to_string(), "http://localhost:3000/baz?foo=baz");
  }
}
