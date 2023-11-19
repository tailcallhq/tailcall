use derive_setters::Setters;
use hyper::HeaderMap;
use iai_callgrind::{black_box, library_benchmark, library_benchmark_group, main};
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
  fn path_string<T: AsRef<str>>(&self, parts: &[T]) -> Option<std::borrow::Cow<'_, str>> {
    self.value.path_string(parts)
  }
}

impl HasHeaders for Context {
  fn headers(&self) -> &HeaderMap {
    &self.headers
  }
}
#[library_benchmark]
fn benchmark_to_request() {
  let tmpl_mustache = RequestTemplate::try_from(Endpoint::new(
    "http://localhost:3000/{{args.b}}?a={{args.a}}&b={{args.b}}&c={{args.c}}".to_string(),
  ))
  .unwrap();

  let tmpl_literal =
    RequestTemplate::try_from(Endpoint::new("http://localhost:3000/foo?a=bar&b=foo&c=baz".to_string())).unwrap();

  let ctx = Context::default().value(json!({
      "args": {
          "b": "foo"
      }
  }));

  // Benchmarks without the criterion framework
  black_box(tmpl_literal.to_request(&ctx).unwrap());
  black_box(tmpl_mustache.to_request(&ctx).unwrap());
}

library_benchmark_group!(
  name= bench_to_request;
  benchmarks= benchmark_to_request);

main!(library_benchmark_groups = bench_to_request);
