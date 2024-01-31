use std::borrow::Cow;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use derive_setters::Setters;
use hyper::HeaderMap;
use serde_json::json;
use tailcall::endpoint::Endpoint;
use tailcall::has_headers::HasHeaders;
use tailcall::http::RequestTemplate;
use tailcall::path::{PathString, PathUrlEncoded};

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
impl PathUrlEncoded for Context {
    fn path_urlencoded<T: AsRef<str>>(&self, path: &[T]) -> Option<Cow<'_, str>> {
        self.value.path_urlencoded(path)
    }
}
impl HasHeaders for Context {
    fn headers(&self) -> &HeaderMap {
        &self.headers
    }
}
fn benchmark_to_request(c: &mut Criterion) {
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

use tailcall::mustache::Mustache;
fn benchmark_set_body(c: &mut Criterion) {
    let tmpl = RequestTemplate::form_encoded_url("http://localhost:3000")
        .unwrap()
        .body_path(Some(Mustache::parse("{{foo.bar}}").unwrap()));
    let ctx = Context::default().value(json!({"foo": {"bar": "baz"}}));
    c.bench_function("with_string", |b| {
        b.iter(|| {
            let _ = black_box(tmpl.to_request(&ctx));
        })
    });

    let tmpl = RequestTemplate::form_encoded_url("http://localhost:3000")
        .unwrap()
        .body_path(Some(Mustache::parse(r#"{"foo": "{{baz}}"}"#).unwrap()));
    let ctx = Context::default().value(json!({"baz": "baz"}));
    c.bench_function("with_json_template", |b| {
        b.iter(|| {
            let _ = black_box(tmpl.to_request(&ctx));
        })
    });

    let tmpl = RequestTemplate::form_encoded_url("http://localhost:3000")
        .unwrap()
        .body_path(Some(Mustache::parse("{{foo}}").unwrap()));
    let ctx = Context::default().value(json!({"foo": {"bar": "baz"}}));
    c.bench_function("with_json_body", |b| {
        b.iter(|| {
            let _ = black_box(tmpl.to_request(&ctx));
        })
    });

    let tmpl = RequestTemplate::form_encoded_url("http://localhost:3000")
        .unwrap()
        .body_path(Some(Mustache::parse("{{a}}").unwrap()));
    let ctx =
        Context::default().value(json!({"a": {"special chars": "a !@#$%^&*()<>?:{}-=1[];',./"}}));
    c.bench_function("with_json_body_nested", |b| {
        b.iter(|| {
            let _ = black_box(tmpl.to_request(&ctx));
        })
    });

    let tmpl = RequestTemplate::form_encoded_url("http://localhost:3000")
        .unwrap()
        .body_path(Some(Mustache::parse(r#"{"foo": "bar"}"#).unwrap()));
    let ctx = Context::default().value(json!({}));
    c.bench_function("with_mustache_literal", |b| {
        b.iter(|| {
            let _ = black_box(tmpl.to_request(&ctx));
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = benchmark_to_request, benchmark_set_body
}
criterion_main!(benches);
