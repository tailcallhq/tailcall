use std::borrow::Cow;
use std::hash::{Hash, Hasher};

use derive_setters::Setters;
use hyper::HeaderMap;
use reqwest::header::HeaderValue;
use tailcall_hasher::TailcallHasher;
use url::Url;

use crate::core::config::Encoding;
use crate::core::endpoint::Endpoint;
use crate::core::has_headers::HasHeaders;
use crate::core::helpers::headers::MustacheHeaders;
use crate::core::ir::{CacheKey, IoId};
use crate::core::mustache::Mustache;
use crate::core::path::PathString;

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
    pub body_path: Option<Mustache>,
    pub endpoint: Endpoint,
    pub encoding: Encoding,
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
            && self.body_path.as_ref().map_or(true, Mustache::is_const)
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
    pub fn to_request<C: PathString + HasHeaders>(
        &self,
        ctx: &C,
    ) -> anyhow::Result<reqwest::Request> {
        // Create url
        let url = self.create_url(ctx)?;
        let method = self.method.clone();
        let mut req = reqwest::Request::new(method, url);
        req = self.set_headers(req, ctx);
        req = self.set_body(req, ctx)?;

        Ok(req)
    }

    /// Sets the body for the request
    fn set_body<C: PathString + HasHeaders>(
        &self,
        mut req: reqwest::Request,
        ctx: &C,
    ) -> anyhow::Result<reqwest::Request> {
        if let Some(body_path) = &self.body_path {
            match &self.encoding {
                Encoding::ApplicationJson => {
                    req.body_mut().replace(body_path.render(ctx).into());
                }
                Encoding::ApplicationXWwwFormUrlencoded => {
                    // TODO: this is a performance bottleneck
                    // We first encode everything to string and then back to form-urlencoded
                    let body: String = body_path.render(ctx);
                    let form_data = match serde_json::from_str::<serde_json::Value>(&body) {
                        Ok(deserialized_data) => serde_urlencoded::to_string(deserialized_data)?,
                        Err(_) => body,
                    };

                    req.body_mut().replace(form_data.into());
                }
            }
        }
        Ok(req)
    }

    /// Sets the headers for the request
    fn set_headers<C: PathString + HasHeaders>(
        &self,
        mut req: reqwest::Request,
        ctx: &C,
    ) -> reqwest::Request {
        let headers = self.create_headers(ctx);
        if !headers.is_empty() {
            req.headers_mut().extend(headers);
        }

        let headers = req.headers_mut();
        // We want to set the header value based on encoding
        // TODO: potential of optimizations.
        // Can set content-type headers while creating the request template
        if self.method != reqwest::Method::GET {
            headers.insert(
                reqwest::header::CONTENT_TYPE,
                match self.encoding {
                    Encoding::ApplicationJson => HeaderValue::from_static("application/json"),
                    Encoding::ApplicationXWwwFormUrlencoded => {
                        HeaderValue::from_static("application/x-www-form-urlencoded")
                    }
                },
            );
        }

        headers.extend(ctx.headers().to_owned());
        req
    }

    pub fn new(root_url: &str) -> anyhow::Result<Self> {
        Ok(Self {
            root_url: Mustache::parse(root_url)?,
            query: Default::default(),
            method: reqwest::Method::GET,
            headers: Default::default(),
            body_path: Default::default(),
            endpoint: Endpoint::new(root_url.to_string()),
            encoding: Default::default(),
        })
    }

    /// Creates a new RequestTemplate with the given form encoded URL
    pub fn form_encoded_url(url: &str) -> anyhow::Result<Self> {
        Ok(Self::new(url)?.encoding(Encoding::ApplicationXWwwFormUrlencoded))
    }

    pub fn with_body(mut self, body: Mustache) -> Self {
        self.body_path = Some(body);
        self
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
        let encoding = endpoint.encoding.clone();

        Ok(Self {
            root_url: path,
            query,
            method,
            headers,
            body_path: body,
            endpoint,
            encoding,
        })
    }
}

impl<Ctx: PathString + HasHeaders> CacheKey<Ctx> for RequestTemplate {
    fn cache_key(&self, ctx: &Ctx) -> Option<IoId> {
        let mut hasher = TailcallHasher::default();
        let state = &mut hasher;

        self.method.hash(state);

        let mut headers = vec![];
        for (name, mustache) in self.headers.iter() {
            name.hash(state);
            mustache.render(ctx).hash(state);
            headers.push((name.to_string(), mustache.render(ctx)));
        }

        for (name, value) in ctx.headers().iter() {
            name.hash(state);
            value.hash(state);
            headers.push((name.to_string(), value.to_str().unwrap().to_string()));
        }

        if let Some(body) = self.body_path.as_ref() {
            body.render(ctx).hash(state)
        }

        let url = self.create_url(ctx).unwrap();
        url.hash(state);

        Some(IoId::new(hasher.finish()))
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
    use crate::core::has_headers::HasHeaders;
    use crate::core::mustache::Mustache;
    use crate::core::path::PathString;

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

    impl crate::core::path::PathString for Context {
        fn path_string<T: AsRef<str>>(&self, parts: &[T]) -> Option<Cow<'_, str>> {
            self.value.path_string(parts)
        }
    }

    impl crate::core::has_headers::HasHeaders for Context {
        fn headers(&self) -> &HeaderMap {
            &self.headers
        }
    }

    impl RequestTemplate {
        fn to_body<C: PathString + HasHeaders>(&self, ctx: &C) -> anyhow::Result<String> {
            let body = self
                .to_request(ctx)?
                .body()
                .and_then(|a| a.as_bytes())
                .map(|a| a.to_vec())
                .unwrap_or_default();

            Ok(std::str::from_utf8(&body)?.to_string())
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
        let tmpl =
            RequestTemplate::new("http://localhost:3000/foo/{{bar.baz}}/boozes/{{bar.booz}}")
                .unwrap();
        let ctx = Context::default().value(json!({
          "bar": {
            "baz": "bar",
            "booz": 1
          }
        }));
        let req = tmpl.to_request(&ctx).unwrap();
        assert_eq!(
            req.url().to_string(),
            "http://localhost:3000/foo/bar/boozes/1"
        );
    }

    #[test]
    fn test_url_query_params() {
        let query = vec![
            ("foo".to_string(), Mustache::parse("0").unwrap()),
            ("bar".to_string(), Mustache::parse("1").unwrap()),
            ("baz".to_string(), Mustache::parse("2").unwrap()),
        ];
        let tmpl = RequestTemplate::new("http://localhost:3000")
            .unwrap()
            .query(query);
        let ctx = Context::default();
        let req = tmpl.to_request(&ctx).unwrap();
        assert_eq!(
            req.url().to_string(),
            "http://localhost:3000/?foo=0&bar=1&baz=2"
        );
    }

    #[test]
    fn test_url_query_params_template() {
        let query = vec![
            ("foo".to_string(), Mustache::parse("0").unwrap()),
            ("bar".to_string(), Mustache::parse("{{bar.id}}").unwrap()),
            ("baz".to_string(), Mustache::parse("{{baz.id}}").unwrap()),
        ];
        let tmpl = RequestTemplate::new("http://localhost:3000/")
            .unwrap()
            .query(query);
        let ctx = Context::default().value(json!({
          "bar": {
            "id": 1
          },
          "baz": {
            "id": 2
          }
        }));
        let req = tmpl.to_request(&ctx).unwrap();
        assert_eq!(
            req.url().to_string(),
            "http://localhost:3000/?foo=0&bar=1&baz=2"
        );
    }

    #[test]
    fn test_headers() {
        let headers = vec![
            (
                HeaderName::from_static("foo"),
                Mustache::parse("foo").unwrap(),
            ),
            (
                HeaderName::from_static("bar"),
                Mustache::parse("bar").unwrap(),
            ),
            (
                HeaderName::from_static("baz"),
                Mustache::parse("baz").unwrap(),
            ),
        ];
        let tmpl = RequestTemplate::new("http://localhost:3000")
            .unwrap()
            .headers(headers);
        let ctx = Context::default();
        let req = tmpl.to_request(&ctx).unwrap();
        assert_eq!(req.headers().get("foo").unwrap(), "foo");
        assert_eq!(req.headers().get("bar").unwrap(), "bar");
        assert_eq!(req.headers().get("baz").unwrap(), "baz");
    }

    #[test]
    fn test_header_template() {
        let headers = vec![
            (
                HeaderName::from_static("foo"),
                Mustache::parse("0").unwrap(),
            ),
            (
                HeaderName::from_static("bar"),
                Mustache::parse("{{bar.id}}").unwrap(),
            ),
            (
                HeaderName::from_static("baz"),
                Mustache::parse("{{baz.id}}").unwrap(),
            ),
        ];
        let tmpl = RequestTemplate::new("http://localhost:3000")
            .unwrap()
            .headers(headers);
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
    fn test_header_encoding_application_json() {
        let tmpl = RequestTemplate::new("http://localhost:3000")
            .unwrap()
            .method(reqwest::Method::POST)
            .encoding(crate::core::config::Encoding::ApplicationJson);
        let ctx = Context::default();
        let req = tmpl.to_request(&ctx).unwrap();
        assert_eq!(
            req.headers().get("Content-Type").unwrap(),
            "application/json"
        );
    }

    #[test]
    fn test_header_encoding_application_x_www_form_urlencoded() {
        let tmpl = RequestTemplate::new("http://localhost:3000")
            .unwrap()
            .method(reqwest::Method::POST)
            .encoding(crate::core::config::Encoding::ApplicationXWwwFormUrlencoded);
        let ctx = Context::default();
        let req = tmpl.to_request(&ctx).unwrap();
        assert_eq!(
            req.headers().get("Content-Type").unwrap(),
            "application/x-www-form-urlencoded"
        );
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
            .body_path(Some(Mustache::parse("foo").unwrap()));
        let ctx = Context::default();
        let body = tmpl.to_body(&ctx).unwrap();
        assert_eq!(body, "foo");
    }

    #[test]
    fn test_body_template() {
        let tmpl = RequestTemplate::new("http://localhost:3000")
            .unwrap()
            .body_path(Some(Mustache::parse("{{foo.bar}}").unwrap()));
        let ctx = Context::default().value(json!({
          "foo": {
            "bar": "baz"
          }
        }));
        let body = tmpl.to_body(&ctx).unwrap();
        assert_eq!(body, "baz");
    }

    #[test]
    fn test_body_encoding_application_json() {
        let tmpl = RequestTemplate::new("http://localhost:3000")
            .unwrap()
            .encoding(crate::core::config::Encoding::ApplicationJson)
            .body_path(Some(Mustache::parse("{{foo.bar}}").unwrap()));
        let ctx = Context::default().value(json!({
          "foo": {
            "bar": "baz"
          }
        }));
        let body = tmpl.to_body(&ctx).unwrap();
        assert_eq!(body, "baz");
    }

    mod endpoint {
        use hyper::HeaderMap;
        use serde_json::json;

        use crate::core::http::request_template::tests::Context;
        use crate::core::http::RequestTemplate;

        #[test]
        fn test_from_endpoint() {
            let mut headers = HeaderMap::new();
            headers.insert("foo", "bar".parse().unwrap());
            let endpoint =
                crate::core::endpoint::Endpoint::new("http://localhost:3000/".to_string())
                    .method(crate::core::http::Method::POST)
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
            let endpoint = crate::core::endpoint::Endpoint::new(
                "http://localhost:3000/{{foo.bar}}".to_string(),
            )
            .method(crate::core::http::Method::POST)
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
            let endpoint = crate::core::endpoint::Endpoint::new(
                "http://localhost:3000/?a={{args.a}}".to_string(),
            );
            let tmpl = RequestTemplate::try_from(endpoint).unwrap();
            let ctx = Context::default();
            let req = tmpl.to_request(&ctx).unwrap();
            assert_eq!(req.url().to_string(), "http://localhost:3000/");
        }

        #[test]
        fn test_from_endpoint_template_with_query_null_value() {
            let endpoint = crate::core::endpoint::Endpoint::new(
                "http://localhost:3000/?a={{args.a}}&q=1".to_string(),
            )
            .query(vec![
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
            let endpoint = crate::core::endpoint::Endpoint::new(
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
            assert_eq!(
                req.url().to_string(),
                "http://localhost:3000/foo?b=foo&d=bar"
            );
        }

        #[test]
        fn test_from_endpoint_template_few_null_value_mixed() {
            let endpoint = crate::core::endpoint::Endpoint::new(
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
            assert_eq!(
                req.url().to_string(),
                "http://localhost:3000/foo?b=foo&d=bar&f=baz"
            );
        }

        #[test]
        fn test_headers_forward() {
            let endpoint =
                crate::core::endpoint::Endpoint::new("http://localhost:3000/".to_string());
            let tmpl = RequestTemplate::try_from(endpoint).unwrap();
            let mut headers = HeaderMap::new();
            headers.insert("baz", "qux".parse().unwrap());
            let ctx = Context::default().headers(headers);
            let req = tmpl.to_request(&ctx).unwrap();
            assert_eq!(req.headers().get("baz").unwrap(), "qux");
        }
    }

    mod form_encoded_url {
        use serde_json::json;

        use crate::core::http::request_template::tests::Context;
        use crate::core::http::RequestTemplate;
        use crate::core::mustache::Mustache;

        #[test]
        fn test_with_string() {
            let tmpl = RequestTemplate::form_encoded_url("http://localhost:3000")
                .unwrap()
                .body_path(Some(Mustache::parse("{{foo.bar}}").unwrap()));
            let ctx = Context::default().value(json!({"foo": {"bar": "baz"}}));
            let request_body = tmpl.to_body(&ctx);
            let body = request_body.unwrap();
            assert_eq!(body, "baz");
        }

        #[test]
        fn test_with_json_template() {
            let tmpl = RequestTemplate::form_encoded_url("http://localhost:3000")
                .unwrap()
                .body_path(Some(Mustache::parse(r#"{"foo": "{{baz}}"}"#).unwrap()));
            let ctx = Context::default().value(json!({"baz": "baz"}));
            let body = tmpl.to_body(&ctx).unwrap();
            assert_eq!(body, "foo=baz");
        }

        #[test]
        fn test_with_json_body() {
            let tmpl = RequestTemplate::form_encoded_url("http://localhost:3000")
                .unwrap()
                .body_path(Some(Mustache::parse("{{foo}}").unwrap()));
            let ctx = Context::default().value(json!({"foo": {"bar": "baz"}}));
            let body = tmpl.to_body(&ctx).unwrap();
            assert_eq!(body, "bar=baz");
        }

        #[test]
        fn test_with_json_body_nested() {
            let tmpl = RequestTemplate::form_encoded_url("http://localhost:3000")
                .unwrap()
                .body_path(Some(Mustache::parse("{{a}}").unwrap()));
            let ctx = Context::default()
                .value(json!({"a": {"special chars": "a !@#$%^&*()<>?:{}-=1[];',./"}}));
            let a = tmpl.to_body(&ctx).unwrap();
            let e = "special+chars=a+%21%40%23%24%25%5E%26*%28%29%3C%3E%3F%3A%7B%7D-%3D1%5B%5D%3B%27%2C.%2F";
            assert_eq!(a, e);
        }

        #[test]
        fn test_with_mustache_literal() {
            let tmpl = RequestTemplate::form_encoded_url("http://localhost:3000")
                .unwrap()
                .body_path(Some(Mustache::parse(r#"{"foo": "bar"}"#).unwrap()));
            let ctx = Context::default().value(json!({}));
            let body = tmpl.to_body(&ctx).unwrap();
            assert_eq!(body, r#"foo=bar"#);
        }
    }

    mod cache_key {
        use std::collections::HashSet;

        use hyper::HeaderMap;
        use serde_json::json;

        use crate::core::http::request_template::tests::Context;
        use crate::core::http::RequestTemplate;
        use crate::core::ir::{CacheKey, IoId};
        use crate::core::mustache::Mustache;

        fn assert_no_duplicate<const N: usize>(arr: [Option<IoId>; N]) {
            let len = arr.len();
            let set = HashSet::from(arr);
            assert_eq!(len, set.len());
        }

        #[test]
        fn test_url_diff() {
            let ctx = Context::default().value(json!({}));
            assert_no_duplicate([
                RequestTemplate::form_encoded_url("http://localhost:3000/1")
                    .unwrap()
                    .cache_key(&ctx),
                RequestTemplate::form_encoded_url("http://localhost:3000/2")
                    .unwrap()
                    .cache_key(&ctx),
                RequestTemplate::form_encoded_url("http://localhost:3001/1")
                    .unwrap()
                    .cache_key(&ctx),
                RequestTemplate::form_encoded_url("http://localhost:3001/2")
                    .unwrap()
                    .cache_key(&ctx),
            ]);
        }

        #[test]
        fn test_headers_diff() {
            let auth_header_ctx = |key, val| {
                let mut headers = HeaderMap::new();
                headers.insert(key, val);
                Context::default().headers(headers)
            };

            assert_no_duplicate([
                RequestTemplate::form_encoded_url("http://localhost:3000")
                    .unwrap()
                    .cache_key(&auth_header_ctx("Authorization", "abc".parse().unwrap())),
                RequestTemplate::form_encoded_url("http://localhost:3000")
                    .unwrap()
                    .cache_key(&auth_header_ctx("Authorization", "bcd".parse().unwrap())),
                RequestTemplate::form_encoded_url("http://localhost:3000")
                    .unwrap()
                    .cache_key(&auth_header_ctx("Range", "bytes=0-100".parse().unwrap())),
                RequestTemplate::form_encoded_url("http://localhost:3000")
                    .unwrap()
                    .cache_key(&auth_header_ctx("Range", "bytes=0-".parse().unwrap())),
            ]);
        }

        #[test]
        fn test_body_diff() {
            let ctx_with_body = |value| Context::default().value(value);

            let key_123_1 = RequestTemplate::form_encoded_url("http://localhost:3000")
                .unwrap()
                .with_body(Mustache::parse("{{args.value}}").unwrap())
                .cache_key(&ctx_with_body(json!({"args": {"value": "123"}})));

            let key_234_1 = RequestTemplate::form_encoded_url("http://localhost:3000")
                .unwrap()
                .with_body(Mustache::parse("{{args.value}}").unwrap())
                .cache_key(&ctx_with_body(json!({"args": {"value": "234"}})));

            let key_123_2 = RequestTemplate::form_encoded_url("http://localhost:3000")
                .unwrap()
                .with_body(Mustache::parse("{{value.id}}").unwrap())
                .cache_key(&ctx_with_body(json!({"value": {"id": "123"}})));

            let key_234_2 = RequestTemplate::form_encoded_url("http://localhost:3000")
                .unwrap()
                .with_body(Mustache::parse("{{value.id2}}").unwrap())
                .cache_key(&ctx_with_body(
                    json!({"value": {"id1": "123", "id2": "234"}}),
                ));

            assert_eq!(key_123_1, key_123_2);
            assert_eq!(key_234_1, key_234_2);
        }
    }
}
