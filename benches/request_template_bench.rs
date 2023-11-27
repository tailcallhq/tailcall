use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serde_json::json;
use tailcall::benchmark::{create_request_templates, Context};

fn benchmark_to_request(c: &mut Criterion) {
  let (tmpl_literal, tmpl_mustache) = create_request_templates();
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

criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = benchmark_to_request
}
criterion_main!(benches);
