use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serde_json::json;

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
                  ]
              },
          ]
      });

      black_box(
        serde_json::to_value(tc_core::json::gather_path_matches(
          &input,
          &["data".into(), "user".into(), "id".into()],
          vec![],
        ))
        .unwrap(),
      );
    })
  });
}

criterion_group!(benches, benchmark_batched_body);
criterion_main!(benches);
