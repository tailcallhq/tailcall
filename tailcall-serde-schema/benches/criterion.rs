use std::collections::HashMap;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use serde::Deserialize;
use tailcall_serde_schema::Schema;

#[derive(Clone, Debug, Deserialize)]
pub struct PostRef<'a> {
    pub user_id: u64,
    pub id: u64,
    #[serde(borrow)]
    pub title: &'a str,
    #[serde(borrow)]
    pub body: &'a str,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Post {
    pub user_id: u64,
    pub id: u64,
    pub title: String,
    pub body: String,
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

fn bench_typed_schema(schema: &Schema) -> serde_json::Value {
    schema.from_str(JSON).unwrap()
}

fn bench_post_deserializer(c: &mut Criterion) {
    let mut group = c.benchmark_group("Deserialization");

    let schema = Schema::array(Schema::object({
        let mut map = HashMap::new();
        map.insert("user_id".to_string(), Schema::u64());
        map.insert("id".to_string(), Schema::u64());
        map.insert("title".to_string(), Schema::String);
        map.insert("body".to_string(), Schema::String);
        map
    }));

    group.bench_function("typed", |b| b.iter(|| black_box(bench_typed())));
    group.bench_function("typed_ref", |b| b.iter(|| black_box(bench_typed_ref())));
    group.bench_function("untyped_ref", |b| b.iter(|| black_box(bench_untyped_ref())));
    group.bench_function("untyped", |b| b.iter(|| black_box(bench_untyped())));
    group.bench_function("typed_schema", |b| {
        b.iter(|| black_box(bench_typed_schema(&schema)))
    });
    group.finish();
}

criterion_group!(benches, bench_post_deserializer);
criterion_main!(benches);
