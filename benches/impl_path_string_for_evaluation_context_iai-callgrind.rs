mod benchmark;

use iai_callgrind::{library_benchmark, library_benchmark_group, main};
use tailcall::http::RequestContext;
use tailcall::lambda::EvaluationContext;
use tailcall::path_string::PathString;

use crate::benchmark::assert_test::{assert_test, MockGraphqlContext, TEST_HEADERS, TEST_VARS};

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

// Define the benchmark function
#[library_benchmark]
fn bench_main() {
  // Initialize the request context with test headers
  let mut req_ctx = RequestContext::default().req_headers(TEST_HEADERS.clone());

  // Set test variables in the request context
  req_ctx.server.vars = TEST_VARS.clone();

  // Create an evaluation context with the request context and mock GraphQL context
  let eval_ctx = EvaluationContext::new(&req_ctx, &MockGraphqlContext);

  // Run the assert_test function to ensure correctness of the EvaluationContext
  assert_test(&eval_ctx);

  // Iterate over all input values and call path_string method
  let all_inputs = INPUT_VALUE
    .iter()
    .chain(ARGS_VALUE)
    .chain(HEADERS_VALUE)
    .chain(VARS_VALUE);

  for input in all_inputs {
    let _result = eval_ctx.path_string(input);
  }
}

// Define the benchmark group
library_benchmark_group!(
    name = bench;
    benchmarks = bench_main
);

// Define the main function for IAI-callgrind
main!(library_benchmark_groups = bench);
