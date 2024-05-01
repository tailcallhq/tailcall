use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use serde_json::json;
use tailcall::blueprint::DynamicValue;
use tailcall::serde_value_ext::ValueExt;

struct BenchData {
    id: String,
    ctx: serde_json::Value,
    dynamic_value: DynamicValue,
}

fn setup() -> Vec<BenchData> {
    let mut data = Vec::new();
    let ctx = json!({"user": {"name": "John Doe","details": {"age": 30,"city": "New York"}}});
    let template = json!({"name": "{{jq: .user.name}}"});
    let dynamic_value = DynamicValue::try_from(&template).unwrap();
    data.push(BenchData { id: "t1".to_string(), ctx, dynamic_value });

    let ctx = json!([{"name": "Alice", "age": 25, "city": "New York"},{"name": "Bob", "age": 30, "city": "Chicago"},{"name": "Sandip", "age": 16, "city": "GoodQuestion"},{"name": "Charlie", "age": 22, "city": "San Francisco"}]);
    let template = json!({"a": "{{jq: sort_by(.age) | .[0].name }}"});
    let dynamic_value = DynamicValue::try_from(&template).unwrap();
    data.push(BenchData { id: "t2".to_string(), ctx, dynamic_value });

    data
}

fn jq_benchmark(c: &mut Criterion) {
    let data = setup();
    for datum in data {
        c.bench_with_input(BenchmarkId::new("data", &datum.id), &datum, |b, datum| {
            b.iter(|| {
                let result = datum.dynamic_value.render_value(&datum.ctx);
                black_box(result).unwrap();
            })
        });
    }
}

criterion_group!(benches, jq_benchmark);
criterion_main!(benches);
