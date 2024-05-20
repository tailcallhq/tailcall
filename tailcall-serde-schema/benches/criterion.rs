use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serde::Deserialize;
use tailcall_serde_schema::{Post, Schema, Value};

#[derive(Clone, Debug, Deserialize)]
pub struct PostRef<'a> {
    pub user_id: u64,
    pub id: u64,
    #[serde(borrow)]
    pub title: &'a str,
    #[serde(borrow)]
    pub body: &'a str,
}

const JSON: &str = include_str!("../data/posts.json");

fn bench_typed_ref() -> Vec<PostRef<'static>> {
    serde_json::from_str(JSON).unwrap()
}

fn bench_typed() -> Vec<Post> {
    serde_json::from_str(JSON).unwrap()
}

fn bench_untyped_ref() -> serde_json_borrow::Value<'static> {
    serde_json::from_str(JSON).unwrap()
}

fn bench_untyped() -> serde_json::Value {
    serde_json::from_str(JSON).unwrap()
}

fn bench_const_value() -> async_graphql::Value {
    serde_json::from_str(JSON).unwrap()
}

fn bench_typed_schema(schema: &Schema) -> Value {
    schema.from_str(JSON).unwrap()
}

fn bench_post_deserializer(c: &mut Criterion) {
    let mut group = c.benchmark_group("Deserialization");

    let schema = Schema::table(&[
        ("user_id", Schema::u64()),
        ("id", Schema::u64()),
        ("title", Schema::string()),
        ("body", Schema::string()),
    ]);

    group.bench_function("typed_schema", |b| {
        b.iter(|| black_box(bench_typed_schema(&schema)))
    });
    group.bench_function("const_value", |b| b.iter(|| black_box(bench_const_value())));
    group.bench_function("typed_ref", |b| b.iter(|| black_box(bench_typed_ref())));
    group.bench_function("untyped_ref", |b| b.iter(|| black_box(bench_untyped_ref())));
    group.bench_function("typed", |b| b.iter(|| black_box(bench_typed())));
    group.bench_function("untyped", |b| b.iter(|| black_box(bench_untyped())));

    group.finish();
}

criterion_group!(benches, bench_post_deserializer);
criterion_main!(benches);
