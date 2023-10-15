use std::borrow::Cow;
use std::collections::HashMap;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use derive_setters::Setters;
use hyper::HeaderMap;
use serde_json::json;
use tailcall::endpoint::Endpoint;
use tailcall::has_headers::HasHeaders;
use tailcall::mustache::Mustache;
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

fn benchmark_to_request_old_vs_new(c: &mut Criterion) {
  let keys: [String; 1000] = core::array::from_fn(|i| "k".to_owned() + &i.to_string());

  let headers: Vec<_> = keys
    .iter()
    .map(|kv| {
      (
        kv.to_owned(),
        Mustache::parse(&("{{h.".to_owned() + kv + "}}")).unwrap(),
      )
    })
    .collect();

  let query = headers.clone();

  let endpoint = Endpoint::new("http://localhost:3000/foo/{{u.a}}/boozes/{{u.v}}".to_string());

  let mut data: HashMap<String, i32> = HashMap::new();
  for (i, k) in keys.into_iter().enumerate() {
    data.insert(k, i.try_into().unwrap());
  }

  let tmpl = RequestTemplate::try_from(endpoint).unwrap();
  let tmpl2 = tmpl.query(query);
  let tmpl3 = tmpl2.headers(headers);
  // tmpl.body(Mustache::parse("{{b.a}}").unwrap());

  let ctx = Context::default().value(json!({
    "h": data.clone(),
    "q" :data.clone(),
    "b" :data.clone(),
    "u":data.clone()
  }));

  c.bench_function("old_to_request", |b| {
    b.iter(|| {
      black_box(tmpl3.to_request(&ctx).unwrap());
    })
  });

  c.bench_function("new_to_request", |b| {
    b.iter(|| {
      black_box(tmpl3.to_request2(&ctx).unwrap());
    })
  });
}

criterion_group!(benches, benchmark_to_request_old_vs_new);
criterion_main!(benches);
