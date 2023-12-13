pub mod benchmark;
use benchmark::assert_test::{assert_test, MockGraphqlContext, TEST_HEADERS, TEST_VARS};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use tailcall::http::RequestContext;
use tailcall::lambda::EvaluationContext;
use tailcall::path::PathString;

// constant input values for benchmarking
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

// Define the benchmark function
fn bench_main(c: &mut Criterion) {
  // Initialize the request context with test headers
  let mut req_ctx = RequestContext::default().req_headers(TEST_HEADERS.clone());

  // Set test variables in the request context
  req_ctx.server.vars = TEST_VARS.clone();

  // Create an evaluation context with the request context and mock GraphQL context
  let eval_ctx = EvaluationContext::new(&req_ctx, &MockGraphqlContext);

  // Run the assert_test function to ensure correctness of the EvaluationContext
  assert_test(&eval_ctx);

  // Iterate over all input values and add benchmarks to Criterion
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

// Define the criterion group
criterion_group!(benches, bench_main);
criterion_main!(benches);
