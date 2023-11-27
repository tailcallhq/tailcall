use iai_callgrind::{library_benchmark, library_benchmark_group, main};
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

#[library_benchmark]
fn bench_main() {
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
    let _result = eval_ctx.path_string(input);
  }
}

library_benchmark_group!(
    name = bench;
    benchmarks = bench_main
);

main!(library_benchmark_groups = bench);
