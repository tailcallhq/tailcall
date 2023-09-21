use derive_setters::Setters;
use hyper::HeaderMap;
use url::Url;

use crate::{endpoint_v2::Endpoint, json::JsonLike, mustache::Mustache};

pub trait AnyPath {
  fn any_path(&self, path: &[String]) -> Option<&str>;
}

impl AnyPath for serde_json::Value {
  fn any_path(&self, path: &[String]) -> Option<&str> {
    self.get_path(path).and_then(|a| a.as_str())
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
    println!("root_url: {}", root_url);
    let url = url::Url::parse(root_url.as_str()).unwrap();
    url
  }
  fn eval_headers<C: AnyPath>(&self, _ctx: &C) -> HeaderMap {
    todo!()
  }
  fn eval_body<C: AnyPath>(&self, _ctx: &C) -> reqwest::Body {
    todo!()
  }

  /// A high-performance way to reliably create a request
  pub fn to_request<C: AnyPath>(self, ctx: &C) -> reqwest::Request {
    let url = self.eval_url(ctx);
    let headers = self.eval_headers(ctx);
    let body = self.eval_body(ctx);
    let method = self.method;

    let mut request = reqwest::Request::new(method, url);
    request.headers_mut().extend(headers);
    request.body_mut().replace(body);

    request
  }

  pub fn new(root_url: &str) -> anyhow::Result<Self> {
    Ok(Self {
      root_url: serde_json::from_str(root_url)?,
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

  mod url {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use crate::request_template::RequestTemplate;

    #[test]
    fn test_url() {
      let tmpl = RequestTemplate::new("http://localhost:3000").unwrap();
      let ctx = serde_json::Value::Null;
      let url = tmpl.eval_url(&ctx);
      assert_eq!(url.to_string(), "http://localhost:3000/");
    }

    #[test]
    fn test_url_path() {
      let tmpl = RequestTemplate::new("foo/bar").unwrap();
      let ctx = serde_json::Value::Null;
      let url = tmpl.eval_url(&ctx);
      assert_eq!(url.to_string(), "foo/bar");
    }

    #[test]
    fn test_url_path_template() {
      let tmpl = RequestTemplate::new("http://localhost:3000/foo/{{bar.baz}}").unwrap();
      let cnt = json!({
        "bar": {
          "baz": "bar"
        }
      });
      let url = tmpl.eval_url(&cnt);
      assert_eq!(url.to_string(), "http://localhost:3000/foo/bar");
    }
  }
}
