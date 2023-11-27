use iai_callgrind::{black_box, library_benchmark, library_benchmark_group, main};
use serde_json::json;
use tailcall::benchmark::{create_request_templates, Context};

#[library_benchmark]
fn benchmark_to_request() {
  let (tmpl_literal, tmpl_mustache) = create_request_templates();
  let ctx = Context::default().value(json!({
      "args": {
          "b": "foo"
      }
  }));

  black_box(tmpl_literal.to_request(&ctx).unwrap());
  black_box(tmpl_mustache.to_request(&ctx).unwrap());
}

library_benchmark_group!(
  name= bench_to_request;
  benchmarks= benchmark_to_request);

main!(library_benchmark_groups = bench_to_request);
