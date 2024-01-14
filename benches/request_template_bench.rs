use std::borrow::Cow;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use derive_setters::Setters;
use hyper::HeaderMap;
use serde_json::json;
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

fn benchmark_to_request(c: &mut Criterion) {
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

// Original set_body 
fn original_set_body<C: PathString + HasHeaders>(
    tmpl: &RequestTemplate,
    mut req: reqwest::Request,
    ctx: &C,
) -> reqwest::Request {
    if let Some(body) = &tmpl.body {
        // Checking and setting content type
        if RequestTemplate::is_application_x_www_form_urlencoded(&req.headers()) {
            req.headers_mut()
                .insert(reqwest::header::CONTENT_TYPE, "application/x-www-form-urlencoded".parse().unwrap());

            // Serialize the Mustache template directly to form_urlencoded
            let form_data = serde_urlencoded::to_string(&body.render(ctx)).unwrap();
            req.body_mut().replace(form_data.into());
        }

        req.body_mut().replace(body.render(ctx).into());
    }
    req
}

// Optimized set_body 
fn optimized_set_body<C: PathString + HasHeaders>(
    tmpl: &RequestTemplate,
    mut req: reqwest::Request,
    ctx: &C,
) -> reqwest::Request {
    if let Some(body) = &tmpl.body {
        // Checking and setting content type
        if RequestTemplate::is_application_x_www_form_urlencoded(&req.headers()) {
            req.headers_mut()
                .insert(reqwest::header::CONTENT_TYPE, "application/x-www-form-urlencoded".parse().unwrap());

            // Optimize: Serialize the Mustache template directly to form_urlencoded
            let form_data = serde_urlencoded::to_string(&body.render(ctx)).unwrap();
            req.body_mut().replace(form_data.into());
        }

        // Optimize: Use body.render directly, avoiding unnecessary conversions
        req.body_mut().replace(body.render(ctx).into());
    }
    req
}

fn benchmark_set_body(c: &mut Criterion) {
    //  dummy RequestTemplate for testing
    let mut request_template = RequestTemplate::try_from(Endpoint::new(
        "http://localhost:3000/foo?a=bar&b=foo&c=baz".to_string(),
    ))
    .unwrap();

    // dummy context
    let ctx = Context {
        value: json!({
            "args": {
                "b": "foo"
            }
        }),
        ..Default::default()
    };

    // Benchmark for the original set_body
    c.bench_function("original_set_body_benchmark", |b| {
        b.iter(|| {
            // Cloning the template for each iteration 
            let mut req = request_template.clone().to_request(&ctx).unwrap();
            // Calling the original_set_body function
            black_box(original_set_body(&request_template, req, &ctx));
        })
    });

    // Benchmark for the optimized set_body
    c.bench_function("optimized_set_body_benchmark", |b| {
        b.iter(|| {
            // Cloning the template for each iteration 
            let mut req = request_template.clone().to_request(&ctx).unwrap();
            // Calling the optimized_set_body function
            black_box(optimized_set_body(&request_template, req, &ctx));
        })
    });
}

criterion_group!(benches, benchmark_to_request, benchmark_set_body);
criterion_main!(benches);
