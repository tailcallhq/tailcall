use std::borrow::Cow;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use derive_setters::Setters;
use hyper::HeaderMap;
use serde_json::json;
use tailcall::config::Encoding;
use tailcall::endpoint::Endpoint;
use tailcall::has_headers::HasHeaders;
use tailcall::http::RequestTemplate;
use tailcall::path::PathString;

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

impl PathString for Context {
  fn path_string<T: AsRef<str>>(&self, parts: &[T]) -> Option<Cow<'_, str>> {
    self.value.path_string(parts)
  }
}

impl HasHeaders for Context {
  fn headers(&self) -> &HeaderMap {
    &self.headers
  }
}

fn initialize_templates() -> (RequestTemplate, RequestTemplate) {
  // Placeholder values, replace with your actual initialization logic
  let tmpl_literal =
    RequestTemplate::try_from(Endpoint::new("http://localhost:3000/foo?a=bar&b=foo&c=baz".to_string())).unwrap();

  let tmpl_mustache = RequestTemplate::try_from(Endpoint::new(
    "http://localhost:3000/{{args.b}}?a={{args.a}}&b={{args.b}}&c={{args.c}}".to_string(),
  ))
  .unwrap();

  (tmpl_literal, tmpl_mustache)
}

fn benchmark_to_request(c: &mut Criterion) {
  let (tmpl_literal, tmpl_mustache) = initialize_templates();

  let ctx = Context::default().value(json!({
      "args": {
          "b": "foo"
      }
  }));

  c.bench_function("with_mustache_literal", |b| {
    b.iter(|| {
      black_box(tmpl_literal.to_request(&ctx).unwrap());
    })
  });

  c.bench_function("with_mustache_expressions", |b| {
    b.iter(|| {
      black_box(tmpl_mustache.to_request(&ctx).unwrap());
    })
  });
}

fn initialize_request_template() -> RequestTemplate {
  // Placeholder value, replace with your actual initialization logic
  RequestTemplate::try_from(Endpoint::new("http://localhost:3000/foo?a=bar&b=foo&c=baz".to_string())).unwrap()
}

// Original set_body function
fn original_set_body<C: PathString + HasHeaders>(
  tmpl: &RequestTemplate,
  mut req: reqwest::Request,
  ctx: &C,
) -> reqwest::Request {
  if let Some(body) = &tmpl.body_path {
    if let Encoding::ApplicationXWwwFormUrlencoded = &tmpl.encoding {
      req.headers_mut().insert(
        reqwest::header::CONTENT_TYPE,
        "application/x-www-form-urlencoded".parse().unwrap(),
      );

      let form_data = serde_urlencoded::to_string(body.render(ctx)).unwrap();
      req.body_mut().replace(form_data.into());
    } else {
      req.body_mut().replace(body.render(ctx).into());
    }
  }
  req
}

// Optimized set_body function
fn optimized_set_body<C: PathString + HasHeaders>(
  tmpl: &RequestTemplate,
  mut req: reqwest::Request,
  ctx: &C,
) -> reqwest::Request {
  if let Some(body) = &tmpl.body_path {
    if let Encoding::ApplicationXWwwFormUrlencoded = &tmpl.encoding {
      req.headers_mut().insert(
        reqwest::header::CONTENT_TYPE,
        "application/x-www-form-urlencoded".parse().unwrap(),
      );

      let form_data = serde_urlencoded::to_string(body.render(ctx)).unwrap();
      req.body_mut().replace(form_data.into());
    } else {
      req.body_mut().replace(body.render(ctx).into());
    }
  }
  req
}

fn benchmark_original_set_body(c: &mut Criterion) {
  let request_template = initialize_request_template();
  let ctx = Context::default().value(json!({"args": {"b": "foo"}}));

  c.bench_function("original_set_body", |b| {
    b.iter(|| {
      let req = request_template.clone().to_request(&ctx).unwrap();
      black_box(original_set_body(&request_template, req, &ctx));
    })
  });
}

// Benchmark the updated optimized set_body function
fn benchmark_optimized_set_body(c: &mut Criterion) {
  let request_template = initialize_request_template();
  let ctx = Context::default().value(json!({"args": {"b": "foo"}}));

  c.bench_function("optimized_set_body", |b| {
    b.iter(|| {
      let req = request_template.clone().to_request(&ctx).unwrap();
      black_box(optimized_set_body(&request_template, req, &ctx));
    })
  });
}

criterion_group!(
  benches,
  benchmark_to_request,
  benchmark_original_set_body,
  benchmark_optimized_set_body
);
criterion_main!(benches);
