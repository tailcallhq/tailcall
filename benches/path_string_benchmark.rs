use criterion::{criterion_group, criterion_main, Criterion};
use tailcall::http::RequestContext;
use tailcall::lambda::EvaluationContext;
use tailcall::path_string::PathString;

fn path_string_for_evolution_context(c: &mut Criterion) {
  let req_ctx = RequestContext::default();
  let eval_ctx = EvaluationContext::new(&req_ctx);

  // Preallocated paths so benchmarks can be mesured ore accuratly
  let value_path = ["value", "a", "b", "c"]
    .iter()
    .map(|&s| s.to_string())
    .collect::<Vec<_>>();
  let not_found_path = ["value", "x", "y", "z"]
    .iter()
    .map(|&s| s.to_string())
    .collect::<Vec<_>>();
  let numeric_path = ["value", "0", "a"]
    .iter()
    .map(|&s| s.to_string())
    .collect::<Vec<_>>();
  let args_path = ["args", "some_arg_key"]
    .iter()
    .map(|&s| s.to_string())
    .collect::<Vec<_>>();
  let deep_nest_path = ["value", "a", "b", "c", "d", "e"]
    .iter()
    .map(|&s| s.to_string())
    .collect::<Vec<_>>();
  let mixed_path = ["value", "args", "headers", "a"]
    .iter()
    .map(|&s| s.to_string())
    .collect::<Vec<_>>();
  let headers_path = ["headers", "Some-Header-Key"]
    .iter()
    .map(|&s| s.to_string())
    .collect::<Vec<_>>();
  let invalid_segments_path = ["invalid", "path", "segments"]
    .iter()
    .map(|&s| s.to_string())
    .collect::<Vec<_>>();
  let special_characters_path = ["valu#e", "ar$gs", "head&ers"]
    .iter()
    .map(|&s| s.to_string())
    .collect::<Vec<_>>();

  c.bench_function("path_string_evolution_context", |b| {
    b.iter(|| {
      let result = eval_ctx.path_string(&value_path);
      criterion::black_box(result);
    })
  });

  c.bench_function("path_string_not_found_evolution_context", |b| {
    b.iter(|| {
      let result = eval_ctx.path_string(&not_found_path);
      criterion::black_box(result);
    })
  });

  c.bench_function("path_string_numeric_evolution_context", |b| {
    b.iter(|| {
      let result = eval_ctx.path_string(&numeric_path);
      criterion::black_box(result);
    })
  });

  c.bench_function("path_string_args_evolution_context", |b| {
    b.iter(|| {
      let result = eval_ctx.path_string(&args_path);
      criterion::black_box(result);
    })
  });

  c.bench_function("path_string_deep_nest_evolution_context", |b| {
    b.iter(|| {
      let result = eval_ctx.path_string(&deep_nest_path);
      criterion::black_box(result);
    })
  });

  c.bench_function("path_string_mixed_evolution_context", |b| {
    b.iter(|| {
      let result = eval_ctx.path_string(&mixed_path);
      criterion::black_box(result);
    })
  });

  c.bench_function("path_string_headers_evolution_context", |b| {
    b.iter(|| {
      let result = eval_ctx.path_string(&headers_path);
      criterion::black_box(result);
    })
  });

  c.bench_function("path_string_invalid_segments_evolution_context", |b| {
    b.iter(|| {
      let result = eval_ctx.path_string(&invalid_segments_path);
      criterion::black_box(result);
    })
  });

  c.bench_function("init_evaluation_context", |b| {
    b.iter(|| {
      let eval_ctx_inner = EvaluationContext::new(&req_ctx);
      criterion::black_box(&eval_ctx_inner);
    })
  });

  c.bench_function("path_string_special_characters_evolution_context", |b| {
    b.iter(|| {
      let result = eval_ctx.path_string(&special_characters_path);
      criterion::black_box(result);
    })
  });
}

criterion_group!(benches, path_string_for_evolution_context);
criterion_main!(benches);
