use std::borrow::Cow;
use std::collections::BTreeSet;
use std::num::NonZeroU64;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use async_graphql::futures_util::future::join_all;
use async_graphql_value::ConstValue;
use criterion::Criterion;
use hyper::body::Bytes;
use reqwest::Request;
use tailcall::core::config::Batch;
use tailcall::core::http::{DataLoaderRequest, HttpDataLoader, Response};
use tailcall::core::ir::IoId;
use tailcall::core::runtime::TargetRuntime;
use tailcall::core::{EnvIO, FileIO, HttpIO};

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
impl EnvIO for Env {
    fn get(&self, _: &str) -> Option<Cow<'_, str>> {
        unimplemented!("Not needed for this bench")
    }
}

struct File;

#[async_trait::async_trait]
impl FileIO for File {
    async fn write<'a>(&'a self, _: &'a str, _: &'a [u8]) -> anyhow::Result<()> {
        unimplemented!("Not needed for this bench")
    }

    async fn read<'a>(&'a self, _: &'a str) -> anyhow::Result<String> {
        unimplemented!("Not needed for this bench")
    }
}

struct Cache;
#[async_trait::async_trait]
impl tailcall::core::Cache for Cache {
    type Key = IoId;
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

pub fn benchmark_data_loader(c: &mut Criterion) {
    c.bench_function("test_data_loader", |b| {
        b.iter(|| {
            let client = Arc::new(MockHttpClient { request_count: Arc::new(AtomicUsize::new(0)) });
            let client_clone = client.clone();
            tokio::runtime::Runtime::new().unwrap().spawn(async move {
                let rt = TargetRuntime {
                    http: client_clone.clone(),
                    http2_only: client_clone,
                    env: Arc::new(Env {}),
                    file: Arc::new(File {}),
                    cache: Arc::new(Cache {}),
                    extensions: Arc::new(vec![]),
                    cmd_worker: None,
                    worker: None,
                };
                let loader = HttpDataLoader::new(rt, None, false);
                let loader = loader.to_data_loader(Batch::default().delay(1));

                let request1 = reqwest::Request::new(
                    reqwest::Method::GET,
                    "http://example.com/1".parse().unwrap(),
                );
                let request2 = reqwest::Request::new(
                    reqwest::Method::GET,
                    "http://example.com/2".parse().unwrap(),
                );

                let headers_to_consider =
                    BTreeSet::from(["Header1".to_string(), "Header2".to_string()]);
                let key1 = DataLoaderRequest::new(request1, headers_to_consider.clone());
                let key2 = DataLoaderRequest::new(request2, headers_to_consider);

                let futures1 = (0..100).map(|_| loader.load_one(key1.clone()));
                let futures2 = (0..100).map(|_| loader.load_one(key2.clone()));
                let _ = join_all(futures1.chain(futures2)).await;
                assert_eq!(
                    client.request_count.load(Ordering::SeqCst),
                    2,
                    "Only one request should be made for the same key"
                );
            })
        })
    });
}
