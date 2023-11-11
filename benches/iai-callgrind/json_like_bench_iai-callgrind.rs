use iai_callgrind::{black_box, library_benchmark, library_benchmark_group, main};
use serde_json::json;

fn gather_path_matches(input: &serde_json::Value, path: &[&str]) -> Option<serde_json::Value> {
  let mut current = input;
  for key in path {
    current = match current.get(key) {
      Some(value) => value,
      None => return None, // Handle the case where the key doesn't exist
    };
  }
  Some(current.clone())
}

#[library_benchmark]
fn benchmark_batched_body() {
  let input = json!({
      "data": [
          {"user": {"id": "1"}},
          {"user": {"id": "2"}},
          {"user": {"id": "3"}},
          {"user": [
              {"id": "4"},
              {"id": "5"}
              ]
          },
      ]
  });

  black_box(gather_path_matches(&input, &["data", "user", "id"]));
}

library_benchmark_group!(
    name= batched_body;
    benchmarks= benchmark_batched_body);

main!(library_benchmark_groups = batched_body);
