use std::borrow::Cow;

use derive_setters::Setters;
use hyper::HeaderMap;
use serde_json::Value;
use url::Url;

use crate::endpoint_v2::Endpoint;
use crate::json::JsonLike;
use crate::mustache_v2::Mustache;

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
  #[allow(dead_code)]
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
  #[allow(dead_code)]
  fn eval_headers<C: AnyPath>(&self, _ctx: &C) -> HeaderMap {
    todo!()
  }
  #[allow(dead_code)]
  fn eval_body<C: AnyPath>(&self, _ctx: &C) -> reqwest::Body {
    todo!()
  }

  /// A high-performance way to reliably create a request
  pub fn to_request<C: AnyPath>(self, ctx: &C) -> reqwest::Request {
    let url = self.eval_url(ctx);
    // let headers = self.eval_headers(ctx);
    // let body = self.eval_body(ctx);
    let method = self.method;

    // request.headers_mut().extend(headers);
    // request.body_mut().replace(body);

    reqwest::Request::new(method, url)
  }

  pub fn new(root_url: &str) -> anyhow::Result<Self> {
    Ok(Self {
      root_url: Mustache::new(root_url)?,
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
    let path = Mustache::new(endpoint.path.as_str())?;
    let query = endpoint
      .query
      .iter()
      .map(|(k, v)| Ok((k.to_owned(), Mustache::new(v.as_str())?)))
      .collect::<anyhow::Result<Vec<_>>>()?;
    let method = endpoint.method.clone().into();
    let headers = endpoint
      .headers
      .iter()
      .map(|(k, v)| Ok((k.as_str().into(), Mustache::new(v.to_str()?)?)))
      .collect::<anyhow::Result<Vec<_>>>()?;

    let body = if let Some(body) = &endpoint.body {
      Some(Mustache::new(body.as_str())?)
    } else {
      None
    };

    Ok(Self { root_url: path, query, method, headers, body })
  }
}

#[cfg(test)]
mod tests {

  use pretty_assertions::assert_eq;
  use serde_json::json;

  use crate::mustache_v2::Mustache;
  use crate::request_template::RequestTemplate;

  #[test]
  fn test_url() {
    let tmpl = RequestTemplate::new(r#""http://localhost:3000""#).unwrap();
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
    let cnt = json!({
      "bar": {
        "baz": "bar"
      }
    });
    let req = tmpl.to_request(&cnt);
    assert_eq!(req.url().to_string(), "http://localhost:3000/foo/bar");
  }
  #[test]
  fn test_url_path_template_multi() {
    let tmpl = RequestTemplate::new(r#""http://localhost:3000/foo/{{bar.baz}}/boozes/{{bar.booz}}""#).unwrap();
    let cnt = json!({
      "bar": {
        "baz": "bar",
        "booz": 1
      }
    });
    let req = tmpl.to_request(&cnt);
    assert_eq!(req.url().to_string(), "http://localhost:3000/foo/bar/boozes/1");
  }
  #[test]
  #[ignore]
  fn test_url_query_params() {
    let query = vec![
      ("foo".to_string(), Mustache::new("0").unwrap()),
      ("bar".to_string(), Mustache::new("1").unwrap()),
      ("baz".to_string(), Mustache::new("2").unwrap()),
    ];
    let tmpl = RequestTemplate::new("http://localhost:3000").unwrap().query(query);
    let ctx = serde_json::Value::Null;
    let req = tmpl.to_request(&ctx);
    assert_eq!(req.url().to_string(), "http://localhost:3000?foo=0&bar=1&baz=2");
  }
  #[test]
  #[ignore]
  fn test_url_query_params_template() {
    let query = vec![
      ("foo".to_string(), Mustache::new("0").unwrap()),
      ("bar".to_string(), Mustache::new("{{bar.id}}").unwrap()),
      ("baz".to_string(), Mustache::new("{{baz.id}}").unwrap()),
    ];
    let tmpl = RequestTemplate::new("http://localhost:3000").unwrap().query(query);
    let ctx = json!({
      "bar": {
        "id": 1
      },
      "baz": {
        "id": 2
      }
    });
    let req = tmpl.to_request(&ctx);
    assert_eq!(req.url().to_string(), "http://localhost:3000?foo=0&bar=1&baz=2");
  }
  #[test]
  #[ignore]
  fn test_headers() {
    let headers = vec![
      ("foo".to_string(), Mustache::new("foo").unwrap()),
      ("bar".to_string(), Mustache::new("bar").unwrap()),
      ("baz".to_string(), Mustache::new("baz").unwrap()),
    ];
    let tmpl = RequestTemplate::new("http://localhost:3000").unwrap().headers(headers);
    let ctx = serde_json::Value::Null;
    let req = tmpl.to_request(&ctx);
    assert_eq!(req.headers().get("foo").unwrap(), "foo");
    assert_eq!(req.headers().get("bar").unwrap(), "bar");
    assert_eq!(req.headers().get("baz").unwrap(), "baz");
  }
  #[test]
  #[ignore]
  fn test_header_template() {
    let headers = vec![
      ("foo".to_string(), Mustache::new("0").unwrap()),
      ("bar".to_string(), Mustache::new("{{bar.id}}").unwrap()),
      ("baz".to_string(), Mustache::new("{{baz.id}}").unwrap()),
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
    assert_eq!(req.headers().get("foo").unwrap(), "foo");
    assert_eq!(req.headers().get("bar").unwrap(), "1");
    assert_eq!(req.headers().get("baz").unwrap(), "2");
  }
  #[test]
  #[ignore]
  fn test_method() {
    let tmpl = RequestTemplate::new("http://localhost:3000")
      .unwrap()
      .method(reqwest::Method::POST);
    let ctx = serde_json::Value::Null;
    let req = tmpl.to_request(&ctx);
    assert_eq!(req.method(), reqwest::Method::POST);
  }
  #[test]
  #[ignore]
  fn test_body() {
    let tmpl = RequestTemplate::new("http://localhost:3000")
      .unwrap()
      .body(Some(Mustache::new("foo").unwrap()));
    let ctx = serde_json::Value::Null;
    let _req = tmpl.to_request(&ctx);
    // assert_eq!(req.body(), "foo");
  }
  #[test]
  #[ignore]
  fn test_body_template() {
    let tmpl = RequestTemplate::new("http://localhost:3000")
      .unwrap()
      .body(Some(Mustache::new("foo.{{bar.baz}}").unwrap()));
    let ctx = json!({
      "foo": {
        "bar": "baz"
      }
    });
    let _req = tmpl.to_request(&ctx);
    // assert_eq!(req.body().unwrap().to_string(), "baz");
  }
}
