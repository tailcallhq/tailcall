use std::borrow::Cow;

use criterion::{black_box, Criterion};
use derive_setters::Setters;
use http::header::HeaderMap;
use serde_json::json;
use tailcall::core::endpoint::Endpoint;
use tailcall::core::has_headers::HasHeaders;
use tailcall::core::http::RequestTemplate;
use tailcall::core::json::JsonLike;
use tailcall::core::path::{PathString, PathValue, ValueString};

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

impl PathValue for Context {
    fn raw_value<'a, T: AsRef<str>>(&'a self, path: &[T]) -> Option<ValueString<'a>> {
        self.value.get_path(path).map(|a| {
            ValueString::Value(Cow::Owned(
                async_graphql::Value::from_json(a.clone()).unwrap(),
            ))
        })
    }
}

impl PathString for Context {
    fn path_string<'a, T: AsRef<str>>(&'a self, parts: &'a [T]) -> Option<Cow<'a, str>> {
        self.value.path_string(parts)
    }
}
impl HasHeaders for Context {
    fn headers(&self) -> &HeaderMap {
        &self.headers
    }
}
pub fn benchmark_to_request(c: &mut Criterion) {
    let tmpl_mustache = RequestTemplate::try_from(Endpoint::new(
        "http://localhost:3000/{{args.b}}?a={{args.a}}&b={{args.b}}&c={{args.c}}".to_string(),
    ))
    .unwrap();

    let tmpl_literal = RequestTemplate::try_from(Endpoint::new(
        "http://localhost:3000/foo?a=bar&b=foo&c=baz".to_string(),
    ))
    .unwrap();

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
