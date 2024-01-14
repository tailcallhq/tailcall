use std::borrow::Cow;

use derive_setters::Setters;
use hyper::HeaderMap;
use reqwest::header::{HeaderValue, CONTENT_TYPE};
use url::Url;
use crate::endpoint::Endpoint;
use crate::has_headers::HasHeaders;
use crate::helpers::headers::MustacheHeaders;
use crate::mustache::Mustache;
use crate::path::PathString;

/// RequestTemplate is an extension of a Mustache template.
/// Various parts of the template can be written as a mustache template.
/// When `to_request` is called, all mustache templates are evaluated.
/// To call `to_request` we need to provide a context.
#[derive(Setters, Debug, Clone)]
pub struct RequestTemplate {
  pub root_url: Mustache,
  pub query: Vec<(String, Mustache)>,
  pub method: reqwest::Method,
  pub headers: MustacheHeaders,
  pub body: Option<Mustache>,
  pub endpoint: Endpoint,
}

impl RequestTemplate {
  /// Creates a URL for the context
  /// Fills in all the mustache templates with required values.
  fn create_url<C: PathString>(&self, ctx: &C) -> anyhow::Result<Url> {
    let mut url = url::Url::parse(self.root_url.render(ctx).as_str())?;
    if self.query.is_empty() && self.root_url.is_const() {
      return Ok(url);
    }
    let extra_qp = self.query.iter().filter_map(|(k, v)| {
      let value = v.render(ctx);
      if value.is_empty() {
        None
      } else {
        Some((Cow::Borrowed(k.as_str()), Cow::Owned(value)))
      }
    });

    let base_qp = url
      .query_pairs()
      .filter_map(|(k, v)| if v.is_empty() { None } else { Some((k, v)) });

    let qp_string = base_qp
      .chain(extra_qp)
      .map(|(k, v)| format!("{}={}", k, v))
      .fold("".to_string(), |str, item| {
        if str.is_empty() {
          item
        } else {
          format!("{}&{}", str, item)
        }
      });

    if qp_string.is_empty() {
      url.set_query(None);
      Ok(url)
    } else {
      url.set_query(Some(qp_string.as_str()));
      Ok(url)
    }
  }

  /// Checks if the template has any mustache templates or not
  /// Returns true if there are not templates
  pub fn is_const(&self) -> bool {
    self.root_url.is_const()
      && self.body.as_ref().map_or(true, Mustache::is_const)
      && self.query.iter().all(|(_, v)| v.is_const())
      && self.headers.iter().all(|(_, v)| v.is_const())
  }

  /// Creates a HeaderMap for the context
  fn create_headers<C: PathString>(&self, ctx: &C) -> HeaderMap {
    let mut header_map = HeaderMap::new();

    for (k, v) in &self.headers {
      if let Ok(header_value) = HeaderValue::from_str(&v.render(ctx)) {
        header_map.insert(k, header_value);
      }
    }

    header_map
  }

  /// Creates a Request for the given context
  pub fn to_request<C: PathString + HasHeaders>(&self, ctx: &C) -> anyhow::Result<reqwest::Request> {
    // Create url
    let url = self.create_url(ctx)?;
    let method = self.method.clone();
    let mut req = reqwest::Request::new(method, url);
    req = self.set_headers(req, ctx);
    req = self.set_body(req, ctx);

    Ok(req)
  }


  /// Sets the body for the request
  pub fn set_body<C: PathString + HasHeaders>(&self, mut req: reqwest::Request, ctx: &C) -> reqwest::Request {
    if let Some(body) = &self.body {
        // Checks and sets content type based on your condition
        if Self::is_application_x_www_form_urlencoded(&req.headers()) {
            req.headers_mut().insert(CONTENT_TYPE, HeaderValue::from_static("application/x-www-form-urlencoded"));

            // Serialize the Mustache template directly to form_urlencoded
            let form_data = serde_urlencoded::to_string(&body.render(ctx)).unwrap();
            req.body_mut().replace(form_data.into());
        }

        req.body_mut().replace(body.render(ctx).into());
    }
    req
}

  /// checks for the CONTENT_TYPE header
  pub fn is_application_x_www_form_urlencoded(headers: &HeaderMap) -> bool {
    if let Some(content_type) = headers.get(CONTENT_TYPE) {
        if let Ok(content_type_str) = content_type.to_str() {
            return content_type_str.to_lowercase().starts_with("application/x-www-form-urlencoded");
        }
    }
    false
  }

  /// Sets the headers for the request
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
    let method = endpoint.method.clone().to_hyper();
    let headers = endpoint
      .headers
      .iter()
      .map(|(k, v)| Ok((k.clone(), Mustache::parse(v.to_str()?)?)))
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

  use derive_setters::Setters;
  use hyper::header::HeaderName;
  use hyper::HeaderMap;
  use pretty_assertions::assert_eq;
  use serde_json::json;

  use super::RequestTemplate;
  use crate::mustache::Mustache;

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
  impl crate::path::PathString for Context {
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
  fn test_url() {
    let tmpl = RequestTemplate::new("http://localhost:3000/").unwrap();
    let ctx = Context::default();
    let req = tmpl.to_request(&ctx).unwrap();
    assert_eq!(req.url().to_string(), "http://localhost:3000/");
  }

  #[test]
  fn test_url_path() {
    let tmpl = RequestTemplate::new("http://localhost:3000/foo/bar").unwrap();
    let ctx = Context::default();
    let req = tmpl.to_request(&ctx).unwrap();
    assert_eq!(req.url().to_string(), "http://localhost:3000/foo/bar");
  }

  #[test]
  fn test_url_path_template() {
    let tmpl = RequestTemplate::new("http://localhost:3000/foo/{{bar.baz}}").unwrap();
    let ctx = Context::default().value(json!({
      "bar": {
        "baz": "bar"
      }
    }));

    let req = tmpl.to_request(&ctx).unwrap();
    assert_eq!(req.url().to_string(), "http://localhost:3000/foo/bar");
  }
  #[test]
  fn test_url_path_template_multi() {
    let tmpl = RequestTemplate::new("http://localhost:3000/foo/{{bar.baz}}/boozes/{{bar.booz}}").unwrap();
    let ctx = Context::default().value(json!({
      "bar": {
        "baz": "bar",
        "booz": 1
      }
    }));
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
    let ctx = Context::default();
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
    let ctx = Context::default().value(json!({
      "bar": {
        "id": 1
      },
      "baz": {
        "id": 2
      }
    }));
    let req = tmpl.to_request(&ctx).unwrap();
    assert_eq!(req.url().to_string(), "http://localhost:3000/?foo=0&bar=1&baz=2");
  }

  #[test]
  fn test_headers() {
    let headers = vec![
      (HeaderName::from_static("foo"), Mustache::parse("foo").unwrap()),
      (HeaderName::from_static("bar"), Mustache::parse("bar").unwrap()),
      (HeaderName::from_static("baz"), Mustache::parse("baz").unwrap()),
    ];
    let tmpl = RequestTemplate::new("http://localhost:3000").unwrap().headers(headers);
    let ctx = Context::default();
    let req = tmpl.to_request(&ctx).unwrap();
    assert_eq!(req.headers().get("foo").unwrap(), "foo");
    assert_eq!(req.headers().get("bar").unwrap(), "bar");
    assert_eq!(req.headers().get("baz").unwrap(), "baz");
  }
  #[test]
  fn test_header_template() {
    let headers = vec![
      (HeaderName::from_static("foo"), Mustache::parse("0").unwrap()),
      (HeaderName::from_static("bar"), Mustache::parse("{{bar.id}}").unwrap()),
      (HeaderName::from_static("baz"), Mustache::parse("{{baz.id}}").unwrap()),
    ];
    let tmpl = RequestTemplate::new("http://localhost:3000").unwrap().headers(headers);
    let ctx = Context::default().value(json!({
      "bar": {
        "id": 1
      },
      "baz": {
        "id": 2
      }
    }));
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
    let ctx = Context::default();
    let req = tmpl.to_request(&ctx).unwrap();
    assert_eq!(req.method(), reqwest::Method::POST);
  }
  #[test]
  fn test_body() {
    let tmpl = RequestTemplate::new("http://localhost:3000")
      .unwrap()
      .body(Some(Mustache::parse("foo").unwrap()));
    let ctx = Context::default();
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
    let ctx = Context::default().value(json!({
      "foo": {
        "bar": "baz"
      }
    }));
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
    let ctx = Context::default();
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
    let ctx = Context::default().value(json!({
      "foo": {
        "bar": "baz",
        "header": "abc"
      }
    }));
    let req = tmpl.to_request(&ctx).unwrap();
    assert_eq!(req.method(), reqwest::Method::POST);
    assert_eq!(req.headers().get("foo").unwrap(), "abc");
    let body = req.body().unwrap().as_bytes().unwrap().to_owned();
    assert_eq!(body, "baz".as_bytes());
    assert_eq!(req.url().to_string(), "http://localhost:3000/baz?foo=baz");
  }

  #[test]
  fn test_from_endpoint_template_null_value() {
    let endpoint = crate::endpoint::Endpoint::new("http://localhost:3000/?a={{args.a}}".to_string());
    let tmpl = RequestTemplate::try_from(endpoint).unwrap();
    let ctx = Context::default();
    let req = tmpl.to_request(&ctx).unwrap();
    assert_eq!(req.url().to_string(), "http://localhost:3000/");
  }

  #[test]
  fn test_from_endpoint_template_with_query_null_value() {
    let endpoint = crate::endpoint::Endpoint::new("http://localhost:3000/?a={{args.a}}&q=1".to_string()).query(vec![
      ("b".to_string(), "1".to_string()),
      ("c".to_string(), "{{args.c}}".to_string()),
    ]);
    let tmpl = RequestTemplate::try_from(endpoint).unwrap();
    let ctx = Context::default();
    let req = tmpl.to_request(&ctx).unwrap();
    assert_eq!(req.url().to_string(), "http://localhost:3000/?q=1&b=1");
  }

  #[test]
  fn test_from_endpoint_template_few_null_value() {
    let endpoint = crate::endpoint::Endpoint::new(
      "http://localhost:3000/{{args.b}}?a={{args.a}}&b={{args.b}}&c={{args.c}}&d={{args.d}}".to_string(),
    );
    let tmpl = RequestTemplate::try_from(endpoint).unwrap();
    let ctx = Context::default().value(json!({
      "args": {
        "b": "foo",
        "d": "bar"
      }
    }));
    let req = tmpl.to_request(&ctx).unwrap();
    assert_eq!(req.url().to_string(), "http://localhost:3000/foo?b=foo&d=bar");
  }

  #[test]
  fn test_from_endpoint_template_few_null_value_mixed() {
    let endpoint = crate::endpoint::Endpoint::new(
      "http://localhost:3000/{{args.b}}?a={{args.a}}&b={{args.b}}&c={{args.c}}&d={{args.d}}".to_string(),
    )
    .query(vec![
      ("e".to_string(), "{{args.e}}".to_string()),
      ("f".to_string(), "{{args.f}}".to_string()),
    ]);
    let tmpl = RequestTemplate::try_from(endpoint).unwrap();
    let ctx = Context::default().value(json!({
      "args": {
        "b": "foo",
        "d": "bar",
        "f": "baz"
      }
    }));
    let req = tmpl.to_request(&ctx).unwrap();
    assert_eq!(req.url().to_string(), "http://localhost:3000/foo?b=foo&d=bar&f=baz");
  }
  #[test]
  fn test_headers_forward() {
    let endpoint = crate::endpoint::Endpoint::new("http://localhost:3000/".to_string());
    let tmpl = RequestTemplate::try_from(endpoint).unwrap();
    let mut headers = HeaderMap::new();
    headers.insert("baz", "qux".parse().unwrap());
    let ctx = Context::default().headers(headers);
    let req = tmpl.to_request(&ctx).unwrap();
    assert_eq!(req.headers().get("baz").unwrap(), "qux");
  }
}
