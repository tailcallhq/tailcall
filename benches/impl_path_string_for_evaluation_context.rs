use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use tailcall::benchmark::{assert_test, MockGraphqlContext, TEST_HEADERS, TEST_VARS};
use tailcall::http::RequestContext;
use tailcall::lambda::EvaluationContext;
use tailcall::path_string::PathString;

const INPUT_VALUE: &[&[&str]] = &[
  // existing values
  &["value", "root"],
  &["value", "nested", "existing"],
  // missing values
  &["value", "missing"],
  &["value", "nested", "missing"],
];

const ARGS_VALUE: &[&[&str]] = &[
  // existing values
  &["args", "root"],
  &["args", "nested", "existing"],
  // missing values
  &["args", "missing"],
  &["args", "nested", "missing"],
];

const HEADERS_VALUE: &[&[&str]] = &[&["headers", "existing"], &["headers", "missing"]];

const VARS_VALUE: &[&[&str]] = &[&["vars", "existing"], &["vars", "missing"]];

fn to_bench_id(input: &[&str]) -> BenchmarkId {
  BenchmarkId::new("input", input.join("."))
}

fn bench_main(c: &mut Criterion) {
  let mut req_ctx = RequestContext::default().req_headers(TEST_HEADERS.clone());

  req_ctx.server.vars = TEST_VARS.clone();
  let eval_ctx = EvaluationContext::new(&req_ctx, &MockGraphqlContext);

  assert_test(&eval_ctx);

  let all_inputs = INPUT_VALUE
    .iter()
    .chain(ARGS_VALUE)
    .chain(HEADERS_VALUE)
    .chain(VARS_VALUE);

  for input in all_inputs {
    c.bench_with_input(to_bench_id(input), input, |b, input| {
      b.iter(|| eval_ctx.path_string(input));
    });
  }
}

criterion_group!(benches, bench_main);
criterion_main!(benches);
