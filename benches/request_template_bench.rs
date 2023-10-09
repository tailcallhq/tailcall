use std::borrow::Cow;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use derive_setters::Setters;
use hyper::HeaderMap;
use serde_json::json;
use tailcall::endpoint::Endpoint;
use tailcall::has_headers::HasHeaders;
use tailcall::path_string::PathString;
use tailcall::request_template::RequestTemplate;

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
fn benchmark_to_request(c: &mut Criterion) {
  c.bench_function("test_to_request", |b| {
    b.iter(|| {
      let endpoint =
        Endpoint::new("http://localhost:3000/{{args.b}}?a={{args.a}}&b={{args.b}}&c={{args.c}}".to_string());
      let tmpl = RequestTemplate::try_from(endpoint).unwrap();
      let ctx = Context::default().value(json!({
        "args": {
          "b": "foo"
        }
      }));
      black_box(tmpl.to_request(&ctx).unwrap());
    })
  });
}

criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = benchmark_to_request
}
criterion_main!(benches);
