use iai_callgrind::{black_box, library_benchmark, library_benchmark_group, main};
use serde_json::json;
use tailcall::benchmark::{create_request_templates, Context};

// Benchmark function for IAI-callgrind
#[library_benchmark]
fn benchmark_to_request() {
  // request templates
  let (tmpl_literal, tmpl_mustache) = create_request_templates();
  // a context with a JSON value
  let ctx = Context::default().value(json!({
      "args": {
          "b": "foo"
      }
  }));

  // Call to_request method for a template with mustache literal expressions
  black_box(tmpl_literal.to_request(&ctx).unwrap());
  // Call to_request method for a template with mustache expressions
  black_box(tmpl_mustache.to_request(&ctx).unwrap());
}

// Define the benchmark group for IAI-callgrind
library_benchmark_group!(
    name= bench_to_request;
    benchmarks= benchmark_to_request
);

// Run the benchmarks for IAI-callgrind
main!(library_benchmark_groups = bench_to_request);
