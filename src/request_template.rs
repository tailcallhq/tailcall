use derive_setters::Setters;
use hyper::HeaderMap;
use reqwest::header::{HeaderName, HeaderValue};
use serde_json::json;
use url::Url;

use crate::endpoint::Endpoint;
use crate::has_headers::HasHeaders;
use crate::mustache::Mustache;
use crate::path_string::PathString;

/// A template to quickly create a request
#[derive(Setters, Debug)]
pub struct RequestTemplate {
  pub root_url: Mustache,
  pub query: Vec<(String, Mustache)>,
  pub method: reqwest::Method,
  pub headers: Vec<(String, Mustache)>,
  pub body: Option<Mustache>,
  pub endpoint: Endpoint,
  static_reqwest: Option<reqwest::Request>,
  p_all_static: bool,
}

impl Clone for RequestTemplate {
  fn clone(&self) -> RequestTemplate {
    RequestTemplate {
      root_url: self.root_url.clone(),
      query: self.query.clone(),
      method: self.method.clone(),
      headers: self.headers.clone(),
      body: self.body.clone(),
      endpoint: self.endpoint.clone(),
      static_reqwest: self.static_reqwest.as_ref().and_then(reqwest::Request::try_clone),
      p_all_static: self.p_all_static,
    }
  }
}

impl RequestTemplate {
  fn eval_url2<C: PathString>(&self, ctx: &C) -> anyhow::Result<Url> {
    let mut url = url::Url::parse(self.root_url.render(ctx).as_str())?;

    if url.query().is_some() {
      let q: Vec<(String, String)> = url
        .query_pairs()
        .filter_map(|(k, v)| {
          if !v.is_empty() {
            Some((k.into_owned(), v.into_owned()))
          } else {
            None
          }
        })
        .collect();

      url.set_query(None);

      if !q.is_empty() {
        url.query_pairs_mut().extend_pairs(q);
      }
    }

    Ok(url)
  }

  #[inline]
  fn static_mustache(v: &Mustache) -> bool {
    v.is_const()
  }

  #[inline]
  fn dynamic_mustache(v: &Mustache) -> bool {
    !v.is_const()
  }

  #[inline]
  fn everything(_: &Mustache) -> bool {
    true
  }

  fn create_static_request(&mut self) {
    let ctx = &json!(null);
    let r = self.setup_req(ctx, RequestTemplate::static_mustache);

    if let Ok(mut req) = r {
      req.headers_mut().insert(
        reqwest::header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
      );
      self.static_reqwest = Some(req);

      self.p_all_static = self.root_url.is_const()
        && self.body.as_ref().map_or(true, Mustache::is_const)
        && self.query.iter().all(|(_, v)| v.is_const())
        && self.headers.iter().all(|(_, v)| v.is_const());
    }
  }

  fn update_query<'a>(url: &mut Url, pairs: impl Iterator<Item = (&'a str, String)>) -> bool {
    let mut p_has_pairs = url.query().is_some();
    let mut query_pairs = url.query_pairs_mut();
    for (k, v) in pairs {
      query_pairs.append_pair(k, &v);
      p_has_pairs = true;
    }
    p_has_pairs
  }

  fn get_req<C: PathString>(&self, ctx: &C) -> anyhow::Result<reqwest::Request> {
    if let Some(r) = &self.static_reqwest {
      let mut req = r.try_clone().unwrap();

      if !self.p_all_static && !self.root_url.is_const() {
        *req.url_mut() = self.eval_url2(ctx)?;
      }

      Ok(req)
    } else {
      self.create_request(ctx)
    }
  }

  fn setup_req<C: PathString>(
    &self,
    ctx: &C,
    mustache_filter: fn(&Mustache) -> bool,
  ) -> anyhow::Result<reqwest::Request> {
    let mut req = self.get_req(ctx)?;

    if self.p_all_static {
      return Ok(req);
    }

    let filter_list = self.query.iter().filter_map(|(k, v)| {
      if mustache_filter(v) {
        let rendered_v = v.render(ctx);
        if !rendered_v.is_empty() {
          Some((k.as_str(), rendered_v))
        } else {
          None
        }
      } else {
        None
      }
    });

    if !RequestTemplate::update_query(req.url_mut(), filter_list) {
      req.url_mut().set_query(None);
    }

    let headers = self.eval_headers2(ctx, mustache_filter);
    if !headers.is_empty() {
      req.headers_mut().extend(headers);
    }

    if let Some(body) = &self.body {
      if mustache_filter(body) {
        req.body_mut().replace(self.eval_body(ctx));
      }
    }

    Ok(req)
  }

  fn create_request<C: PathString>(&self, ctx: &C) -> anyhow::Result<reqwest::Request> {
    let url = self.eval_url2(ctx)?;
    let method = self.method.clone();
    Ok(reqwest::Request::new(method, url))
  }

  fn eval_headers2<C: PathString>(&self, ctx: &C, mustache_filter: fn(&Mustache) -> bool) -> HeaderMap {
    let mut header_map = HeaderMap::new();

    for (k, v) in &self.headers {
      if mustache_filter(v) {
        if let Ok(header_name) = HeaderName::from_bytes(k.as_bytes()) {
          if let Ok(header_value) = HeaderValue::from_str(&v.render(ctx)) {
            header_map.insert(header_name, header_value);
          }
        }
      }
    }

    header_map
  }

  fn eval_url<C: PathString>(&self, ctx: &C) -> anyhow::Result<Url> {
    let root_url = self.root_url.render(ctx);
    let mut url = url::Url::parse(root_url.as_str())?;
    url
      .query_pairs_mut()
      .extend_pairs(self.query.iter().map(|(k, v)| (k.as_str(), v.render(ctx))));

    let query_string = url
      .query_pairs()
      .filter_map(|(k, v)| if v.is_empty() { None } else { Some((k, v)) })
      .map(|(k, v)| format!("{}={}", k, v))
      .collect::<Vec<_>>()
      .join("&");

    if !query_string.is_empty() {
      url.set_query(Some(&query_string));
    } else {
      url.set_query(None);
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
      .map_or(reqwest::Body::from("".to_string()), |b| b.render(ctx).into())
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

  /// A high-performance way to reliably create a request
  pub fn to_request2<C: PathString + HasHeaders>(&self, ctx: &C) -> anyhow::Result<reqwest::Request> {
    if self.static_reqwest.is_some() {
      let mut req = self.setup_req(ctx, RequestTemplate::dynamic_mustache)?;

      let ctx_headers = ctx.headers().to_owned();

      if !ctx_headers.is_empty() {
        req.headers_mut().extend(ctx_headers);
      }

      Ok(req)
    } else {
      let mut req = self.setup_req(ctx, RequestTemplate::everything)?;
      let headers = req.headers_mut();
      headers.insert(
        reqwest::header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
      );
      headers.extend(ctx.headers().to_owned());
      Ok(req)
    }
  }

  pub fn new(root_url: &str) -> anyhow::Result<Self> {
    Ok(Self {
      root_url: Mustache::parse(root_url)?,
      query: Default::default(),
      method: reqwest::Method::GET,
      headers: Default::default(),
      body: Default::default(),
      endpoint: Endpoint::new(root_url.to_string()),
      static_reqwest: None,
      p_all_static: false,
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

    let mut req =
      Self { root_url: path, query, method, headers, body, endpoint, static_reqwest: None, p_all_static: false };
    req.create_static_request();
    Ok(req)
  }
}

#[cfg(test)]
mod tests {
  use std::borrow::Cow;

  use derive_setters::Setters;
  use hyper::HeaderMap;
  use pretty_assertions::assert_eq;
  use serde_json::json;

  use crate::mustache::Mustache;
  use crate::request_template::RequestTemplate;

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
      ("foo".to_string(), Mustache::parse("foo").unwrap()),
      ("bar".to_string(), Mustache::parse("bar").unwrap()),
      ("baz".to_string(), Mustache::parse("baz").unwrap()),
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
      ("foo".to_string(), Mustache::parse("0").unwrap()),
      ("bar".to_string(), Mustache::parse("{{bar.id}}").unwrap()),
      ("baz".to_string(), Mustache::parse("{{baz.id}}").unwrap()),
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

  //testing to_request2 should be removed when to_request2 replaces to_request
  #[test]
  fn test_url2() {
    let tmpl = RequestTemplate::new("http://localhost:3000/").unwrap();
    let ctx = Context::default();
    let req = tmpl.to_request2(&ctx).unwrap();
    assert_eq!(req.url().to_string(), "http://localhost:3000/");
  }

  #[test]
  fn test_url_path2() {
    let tmpl = RequestTemplate::new("http://localhost:3000/foo/bar").unwrap();
    let ctx = Context::default();
    let req = tmpl.to_request2(&ctx).unwrap();
    assert_eq!(req.url().to_string(), "http://localhost:3000/foo/bar");
  }

  #[test]
  fn test_url_path_template2() {
    let tmpl = RequestTemplate::new("http://localhost:3000/foo/{{bar.baz}}").unwrap();
    let ctx = Context::default().value(json!({
      "bar": {
        "baz": "bar"
      }
    }));

    let req = tmpl.to_request2(&ctx).unwrap();
    assert_eq!(req.url().to_string(), "http://localhost:3000/foo/bar");
  }
  #[test]
  fn test_url_path_template_multi2() {
    let tmpl = RequestTemplate::new("http://localhost:3000/foo/{{bar.baz}}/boozes/{{bar.booz}}").unwrap();
    let ctx = Context::default().value(json!({
      "bar": {
        "baz": "bar",
        "booz": 1
      }
    }));
    let req = tmpl.to_request2(&ctx).unwrap();
    assert_eq!(req.url().to_string(), "http://localhost:3000/foo/bar/boozes/1");
  }
  #[test]
  fn test_url_query_params2() {
    let query = vec![
      ("foo".to_string(), Mustache::parse("0").unwrap()),
      ("bar".to_string(), Mustache::parse("1").unwrap()),
      ("baz".to_string(), Mustache::parse("2").unwrap()),
    ];
    let tmpl = RequestTemplate::new("http://localhost:3000").unwrap().query(query);
    let ctx = Context::default();
    let req = tmpl.to_request2(&ctx).unwrap();
    assert_eq!(req.url().to_string(), "http://localhost:3000/?foo=0&bar=1&baz=2");
  }
  #[test]
  fn test_url_query_params_template2() {
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
    let req = tmpl.to_request2(&ctx).unwrap();
    assert_eq!(req.url().to_string(), "http://localhost:3000/?foo=0&bar=1&baz=2");
  }
  #[test]
  fn test_headers2() {
    let headers = vec![
      ("foo".to_string(), Mustache::parse("foo").unwrap()),
      ("bar".to_string(), Mustache::parse("bar").unwrap()),
      ("baz".to_string(), Mustache::parse("baz").unwrap()),
    ];
    let tmpl = RequestTemplate::new("http://localhost:3000").unwrap().headers(headers);
    let ctx = Context::default();
    let req = tmpl.to_request2(&ctx).unwrap();
    assert_eq!(req.headers().get("foo").unwrap(), "foo");
    assert_eq!(req.headers().get("bar").unwrap(), "bar");
    assert_eq!(req.headers().get("baz").unwrap(), "baz");
  }
  #[test]
  fn test_header_template2() {
    let headers = vec![
      ("foo".to_string(), Mustache::parse("0").unwrap()),
      ("bar".to_string(), Mustache::parse("{{bar.id}}").unwrap()),
      ("baz".to_string(), Mustache::parse("{{baz.id}}").unwrap()),
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
    let req = tmpl.to_request2(&ctx).unwrap();
    assert_eq!(req.headers().get("foo").unwrap(), "0");
    assert_eq!(req.headers().get("bar").unwrap(), "1");
    assert_eq!(req.headers().get("baz").unwrap(), "2");
  }
  #[test]
  fn test_method2() {
    let tmpl = RequestTemplate::new("http://localhost:3000")
      .unwrap()
      .method(reqwest::Method::POST);
    let ctx = Context::default();
    let req = tmpl.to_request2(&ctx).unwrap();
    assert_eq!(req.method(), reqwest::Method::POST);
  }
  #[test]
  fn test_body2() {
    let tmpl = RequestTemplate::new("http://localhost:3000")
      .unwrap()
      .body(Some(Mustache::parse("foo").unwrap()));
    let ctx = Context::default();
    let body = tmpl
      .to_request2(&ctx)
      .unwrap()
      .body()
      .unwrap()
      .as_bytes()
      .unwrap()
      .to_owned();
    assert_eq!(body, "foo".as_bytes());
  }
  #[test]
  fn test_body_template2() {
    let tmpl = RequestTemplate::new("http://localhost:3000")
      .unwrap()
      .body(Some(Mustache::parse("{{foo.bar}}").unwrap()));
    let ctx = Context::default().value(json!({
      "foo": {
        "bar": "baz"
      }
    }));
    let body = tmpl
      .to_request2(&ctx)
      .unwrap()
      .body()
      .unwrap()
      .as_bytes()
      .unwrap()
      .to_owned();
    assert_eq!(body, "baz".as_bytes());
  }
  #[test]
  fn test_from_endpoint2() {
    let mut headers = HeaderMap::new();
    headers.insert("foo", "bar".parse().unwrap());
    let endpoint = crate::endpoint::Endpoint::new("http://localhost:3000/".to_string())
      .method(crate::http::Method::POST)
      .headers(headers)
      .body(Some("foo".into()));
    let tmpl = RequestTemplate::try_from(endpoint).unwrap();
    let ctx = Context::default();
    let req = tmpl.to_request2(&ctx).unwrap();
    assert_eq!(req.method(), reqwest::Method::POST);
    assert_eq!(req.headers().get("foo").unwrap(), "bar");
    let body = req.body().unwrap().as_bytes().unwrap().to_owned();
    assert_eq!(body, "foo".as_bytes());
    assert_eq!(req.url().to_string(), "http://localhost:3000/");
  }
  #[test]
  fn test_from_endpoint_template2() {
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
    let req = tmpl.to_request2(&ctx).unwrap();
    assert_eq!(req.method(), reqwest::Method::POST);
    assert_eq!(req.headers().get("foo").unwrap(), "abc");
    let body = req.body().unwrap().as_bytes().unwrap().to_owned();
    assert_eq!(body, "baz".as_bytes());
    assert_eq!(req.url().to_string(), "http://localhost:3000/baz?foo=baz");
  }

  #[test]
  fn test_from_endpoint_template_null_value2() {
    let endpoint = crate::endpoint::Endpoint::new("http://localhost:3000/?a={{args.a}}".to_string());
    let tmpl = RequestTemplate::try_from(endpoint).unwrap();
    let ctx = Context::default();
    let req = tmpl.to_request2(&ctx).unwrap();
    assert_eq!(req.url().to_string(), "http://localhost:3000/");
  }

  #[test]
  fn test_from_endpoint_template_few_null_value2() {
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
    let req = tmpl.to_request2(&ctx).unwrap();
    assert_eq!(req.url().to_string(), "http://localhost:3000/foo?b=foo&d=bar");
  }

  #[test]
  fn test_from_endpoint_template_few_null_value_mixed2() {
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
    let req = tmpl.to_request2(&ctx).unwrap();
    assert_eq!(req.url().to_string(), "http://localhost:3000/foo?b=foo&d=bar&f=baz");
  }
  #[test]
  fn test_headers_forward2() {
    let endpoint = crate::endpoint::Endpoint::new("http://localhost:3000/".to_string());
    let tmpl = RequestTemplate::try_from(endpoint).unwrap();
    let mut headers = HeaderMap::new();
    headers.insert("baz", "qux".parse().unwrap());
    let ctx = Context::default().headers(headers);
    let req = tmpl.to_request2(&ctx).unwrap();
    assert_eq!(req.headers().get("baz").unwrap(), "qux");
  }
}
