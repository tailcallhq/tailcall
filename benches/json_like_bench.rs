use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serde_json::json;
use tailcall::benchmark::gather_path_matches;

fn benchmark_batched_body(c: &mut Criterion) {
  c.bench_function("test_batched_body", |b| {
    b.iter(|| {
      let input = json!({
          "data": [
              {"user": {"id": "1"}},
              {"user": {"id": "2"}},
              {"user": {"id": "3"}},
              {"user": [
                  {"id": "4"},
                  {"id": "5"}
              ]}
          ]
      });

      // Use the gather_path_matches function
      black_box(serde_json::to_value(gather_path_matches(&input, &["data", "user", "id"])).unwrap());
    })
  });
}

criterion_group!(benches, benchmark_batched_body);
criterion_main!(benches);
