use std::borrow::Cow;
use std::num::NonZeroU64;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use async_graphql_value::ConstValue;
use async_trait::async_trait;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hyper::body::Bytes;
use reqwest::Request;
use serde_json::json;
use tailcall::blueprint::DynamicValue;
use tailcall::http::{RequestContext, Response};
use tailcall::{EnvIO, FileIO, HttpIO};
use tailcall::cache::InMemoryCache;
use tailcall::lambda::{Eval, EvaluationContext, Expression};
use tailcall::runtime::TargetRuntime;
use tailcall::serde_value_ext::ValueExt;
use tailcall::valid::Valid;

#[derive(Clone)]
struct MockHttpClient {
    // To keep track of number of times execute is called
    request_count: Arc<AtomicUsize>,
}

#[async_trait::async_trait]
impl HttpIO for MockHttpClient {
    async fn execute(&self, _req: Request) -> anyhow::Result<Response<Bytes>> {
        Ok(Response::empty())
    }
}

struct Env {}
#[async_trait]
impl EnvIO for Env {
    fn get(&self, _: &str) -> Option<Cow<'_, str>> {
        unimplemented!("Not needed for this bench")
    }
}

struct File;

#[async_trait]
impl FileIO for File {
    async fn write<'a>(&'a self, _: &'a str, _: &'a [u8]) -> anyhow::Result<()> {
        unimplemented!("Not needed for this bench")
    }

    async fn read<'a>(&'a self, _: &'a str) -> anyhow::Result<String> {
        unimplemented!("Not needed for this bench")
    }
}

struct Cache;
#[async_trait]
impl tailcall::Cache for Cache {
    type Key = u64;
    type Value = ConstValue;

    async fn set<'a>(&'a self, _: Self::Key, _: Self::Value, _: NonZeroU64) -> anyhow::Result<()> {
        unimplemented!("Not needed for this bench")
    }

    async fn get<'a>(&'a self, _: &'a Self::Key) -> anyhow::Result<Option<Self::Value>> {
        unimplemented!("Not needed for this bench")
    }

    fn hit_rate(&self) -> Option<f64> {
        unimplemented!("Not needed for this bench")
    }
}

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
    let mut defs = jaq_interpret::ParseCtx::new(vec![]);
    defs.insert_natives(jaq_core::core());
    defs.insert_defs(jaq_std::std());

    let filter = ".foo.bar.baz";
    let (filter, errs) = jaq_parse::parse(filter, jaq_parse::main());
    let errs = errs
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<String>>()
        .join("\n");
    let http = Arc::new(Http::init());
    let http2 = Arc::new(Http::init(&upstream.clone().http2_only(true)));
    let runtime = TargetRuntime {
        http2_only: http2,
        http,
        env: Arc::new(Env {}),
        file: Arc::new(File {}),
        cache: Arc::new(InMemoryCache::new()),
        extensions: Arc::new(vec![]),
    };
    let jq = Expression::Jq(defs.compile(filter.unwrap()));
    jq.eval(EvaluationContext::new(RequestContext::new(TargetRuntime {

    }))).unwrap();
    c.bench_function("jq_bench", |b| {
        b.iter(|| {
            black_box(dynamic_value.render_value(&data)).unwrap();
        })
    });
}

criterion_group!(benches, bench_jq_and_mustache);
criterion_main!(benches);