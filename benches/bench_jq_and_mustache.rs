use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serde_json::json;
use tailcall::blueprint::DynamicValue;
use tailcall::serde_value_ext::ValueExt;

fn test_data() -> serde_json::Value {
    json!({"foo": {"bar": {"baz": 1}}})
}
fn bench_jq_and_mustache(c: &mut Criterion) {
    let data = test_data();
    let value = json!({"a": "{{.foo.bar.baz}}"});
    let dynamic_value = DynamicValue::try_from(&value).unwrap();
    c.bench_function("mustache_bench", |b| {
        b.iter(|| {
            black_box(dynamic_value.render_value(&data)).unwrap();
        })
    });
    let data = test_data();
    let value = json!({"a": "{{jq: .foo.bar.baz}}"});
    let dynamic_value = DynamicValue::try_from(&value).unwrap();
    c.bench_function("jq_bench", |b| {
        b.iter(|| {
            black_box(dynamic_value.render_value(&data)).unwrap();
        })
    });
}

criterion_group!(benches, bench_jq_and_mustache);
criterion_main!(benches);
