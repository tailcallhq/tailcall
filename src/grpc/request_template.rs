use std::borrow::Cow;

use derive_setters::Setters;
use hyper::header::CONTENT_TYPE;
use hyper::{HeaderMap, Method};
use reqwest::header::HeaderValue;
use url::Url;

use crate::grpc::protobuf::ProtobufOperation;
use crate::has_headers::HasHeaders;
use crate::helpers::headers::MustacheHeaders;
use crate::mustache::Mustache;
use crate::path::PathString;

const GRPC_MIME_TYPE: HeaderValue = HeaderValue::from_static("application/grpc");

#[derive(Setters, Debug, Clone)]
pub struct RequestTemplate {
  pub url: Mustache,
  pub headers: MustacheHeaders,
  pub body: Option<Mustache>,
  pub operation: ProtobufOperation,
}

impl RequestTemplate {
  fn create_url<C: PathString>(&self, ctx: &C) -> anyhow::Result<Url> {
    let url = url::Url::parse(self.url.render(ctx).as_str())?;

    Ok(url)
  }

  fn create_headers<C: PathString>(&self, ctx: &C) -> HeaderMap {
    let mut header_map = HeaderMap::new();

    header_map.insert(CONTENT_TYPE, GRPC_MIME_TYPE);

    for (k, v) in &self.headers {
      if let Ok(header_value) = HeaderValue::from_str(&v.render(ctx)) {
        header_map.insert(k, header_value);
      }
    }

    header_map
  }

  pub fn to_request<C: PathString + HasHeaders>(&self, ctx: &C) -> anyhow::Result<reqwest::Request> {
    let url = self.create_url(ctx)?;
    let mut req = reqwest::Request::new(Method::POST, url);
    req = self.set_headers(req, ctx);
    req = self.set_body(req, ctx)?;
    *req.version_mut() = reqwest::Version::HTTP_2;
    Ok(req)
  }

  fn set_body<C: PathString + HasHeaders>(
    &self,
    mut req: reqwest::Request,
    ctx: &C,
  ) -> anyhow::Result<reqwest::Request> {
    let input = if let Some(body) = &self.body {
      Cow::Owned(body.render(ctx))
    } else {
      Cow::Borrowed("{}")
    };
    let body = self.operation.convert_input(&input)?;

    req.body_mut().replace(body.into());

    Ok(req)
  }

  fn set_headers<C: PathString + HasHeaders>(&self, mut req: reqwest::Request, ctx: &C) -> reqwest::Request {
    let req_headers = req.headers_mut();

    let headers = self.create_headers(ctx);
    if !headers.is_empty() {
      req_headers.extend(headers);
    }

    req_headers.extend(ctx.headers().to_owned());
    req
  }
}

#[cfg(test)]
mod tests {
  use std::{borrow::Cow, path::PathBuf};

  use derive_setters::Setters;
  use hyper::{
    header::{HeaderName, HeaderValue},
    HeaderMap, Method, Version,
  };
  use once_cell::sync::Lazy;
  use pretty_assertions::assert_eq;

  use super::RequestTemplate;
  use crate::{
    grpc::protobuf::{ProtobufOperation, ProtobufService, ProtobufSet},
    mustache::Mustache,
  };

  static PROTOBUF_OPERATION: Lazy<ProtobufOperation> = Lazy::new(|| {
    let root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut test_file = root_dir.join(file!());

    test_file.pop();
    test_file.push("tests");
    test_file.push("greetings.proto");

    let protobuf_set = ProtobufSet::from_proto_file(&test_file).unwrap();
    let service = ProtobufService::new(&protobuf_set, "Greeter").unwrap();

    ProtobufOperation::new(&service, "SayHello").unwrap()
  });

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
  fn request_with_empty_body() {
    let tmpl = RequestTemplate {
      url: Mustache::parse("http://localhost:3000/").unwrap(),
      headers: vec![(
        HeaderName::from_static("test-header"),
        Mustache::parse("value").unwrap(),
      )],
      operation: PROTOBUF_OPERATION.clone(),
      body: None,
    };
    let ctx = Context::default();
    let req = tmpl.to_request(&ctx).unwrap();

    assert_eq!(req.url().as_str(), "http://localhost:3000/");
    assert_eq!(req.method(), Method::POST);
    assert_eq!(req.version(), Version::HTTP_2);
    assert_eq!(
      req.headers(),
      &HeaderMap::from_iter([
        (
          HeaderName::from_static("test-header"),
          HeaderValue::from_static("value")
        ),
        (
          HeaderName::from_static("content-type"),
          HeaderValue::from_static("application/grpc")
        )
      ])
    );

    req.body().map(|body| assert_eq!(body.as_bytes(), Some(b"\0\0\0\0\0".as_ref())));
  }

  #[test]
  fn request_with_body() {
    let tmpl = RequestTemplate {
      url: Mustache::parse("http://localhost:3000/").unwrap(),
      headers: vec![],
      operation: PROTOBUF_OPERATION.clone(),
      body: Some(Mustache::parse(r#"{ "name": "test" }"#).unwrap()),
    };
    let ctx = Context::default();
    let req = tmpl.to_request(&ctx).unwrap();

    req.body().map(|body| assert_eq!(body.as_bytes(), Some(b"\0\0\0\0\x06\n\x04test".as_ref())));
  }
}
