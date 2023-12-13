pub mod benchmark;
use benchmark::create_request_templates::{create_request_templates, Context};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serde_json::json;

// Benchmark function to test the performance of to_request method
fn benchmark_to_request(c: &mut Criterion) {
  // request templates
  let (tmpl_literal, tmpl_mustache) = create_request_templates();
  // a context with a JSON value
  let ctx = Context::default().value(json!({
      "args": {
          "b": "foo"
      }
  }));

  // Benchmark to_request method for a template with mustache literal expressions
  c.bench_function("with_mustache_literal", |b| {
    b.iter(|| {
      black_box(tmpl_literal.to_request(&ctx).unwrap());
    })
  });

  // Benchmark to_request method for a template with mustache expressions
  c.bench_function("with_mustache_expressions", |b| {
    b.iter(|| {
      black_box(tmpl_mustache.to_request(&ctx).unwrap());
    })
  });
}

// criterion group for the benchmark
criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = benchmark_to_request
}

// Run the benchmarks
criterion_main!(benches);
