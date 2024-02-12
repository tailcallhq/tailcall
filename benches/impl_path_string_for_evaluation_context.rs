use std::borrow::Cow;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use async_graphql::context::SelectionField;
use async_graphql::{Name, Value};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use hyper::header::HeaderValue;
use hyper::HeaderMap;
use indexmap::IndexMap;
use once_cell::sync::Lazy;
use tailcall::http::RequestContext;
use tailcall::lambda::{EvaluationContext, ResolverContextLike};
use tailcall::path::PathString;

#[cfg(test)]
pub mod test {
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::Duration;

    use anyhow::{anyhow, Result};
    use http_cache_reqwest::{Cache, CacheMode, HttpCache, HttpCacheOptions, MokaManager};
    use hyper::body::Bytes;
    use reqwest::Client;
    use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
    use tailcall::cache::InMemoryCache;
    use tailcall::http::Response;
    use tailcall::runtime::TargetRuntime;
    use tailcall::{EnvIO, FileIO, HttpIO};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[derive(Clone)]
    struct TestHttp {
        client: ClientWithMiddleware,
    }

    impl Default for TestHttp {
        fn default() -> Self {
            Self { client: ClientBuilder::new(Client::new()).build() }
        }
    }

    impl TestHttp {
        fn init(h2only: bool) -> Self {
            let mut builder = Client::builder()
                .tcp_keepalive(Some(Duration::from_secs(5)))
                .timeout(Duration::from_secs(60))
                .connect_timeout(Duration::from_secs(60))
                .http2_keep_alive_interval(Some(Duration::from_secs(60)))
                .http2_keep_alive_timeout(Duration::from_secs(60))
                .http2_keep_alive_while_idle(false)
                .pool_idle_timeout(Some(Duration::from_secs(60)))
                .pool_max_idle_per_host(60)
                .user_agent("Tailcall/1.0".to_string());

            // Add Http2 Prior Knowledge
            if h2only {
                builder = builder.http2_prior_knowledge();
            }

            let mut client = ClientBuilder::new(builder.build().expect("Failed to build client"));

            client = client.with(Cache(HttpCache {
                mode: CacheMode::Default,
                manager: MokaManager::default(),
                options: HttpCacheOptions::default(),
            }));

            Self { client: client.build() }
        }
    }

    #[async_trait::async_trait]
    impl HttpIO for TestHttp {
        async fn execute(&self, request: reqwest::Request) -> Result<Response<Bytes>> {
            let response = self.client.execute(request).await;
            Response::from_reqwest(response?.error_for_status()?).await
        }
    }

    #[derive(Clone)]
    struct TestFileIO {}

    impl TestFileIO {
        fn init() -> Self {
            TestFileIO {}
        }
    }

    #[async_trait::async_trait]
    impl FileIO for TestFileIO {
        async fn write<'a>(&'a self, path: &'a str, content: &'a [u8]) -> anyhow::Result<()> {
            let mut file = tokio::fs::File::create(path).await?;
            file.write_all(content)
                .await
                .map_err(|e| anyhow!("{}", e))?;
            Ok(())
        }

        async fn read<'a>(&'a self, path: &'a str) -> anyhow::Result<String> {
            let mut file = tokio::fs::File::open(path).await?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)
                .await
                .map_err(|e| anyhow!("{}", e))?;
            Ok(String::from_utf8(buffer)?)
        }
    }

    #[derive(Clone)]
    struct TestEnvIO {
        vars: HashMap<String, String>,
    }

    impl EnvIO for TestEnvIO {
        fn get(&self, key: &str) -> Option<String> {
            self.vars.get(key).cloned()
        }
    }

    impl TestEnvIO {
        pub fn init() -> Self {
            Self { vars: std::env::vars().collect() }
        }
    }

    pub fn init() -> TargetRuntime {
        let http: Arc<dyn HttpIO + Sync + Send> = Arc::new(TestHttp::init(false));

        let http2: Arc<dyn HttpIO + Sync + Send> = Arc::new(TestHttp::init(true));

        let file = TestFileIO::init();
        let env = TestEnvIO::init();

        TargetRuntime {
            http,
            http2_only: http2,
            env: Arc::new(env),
            file: Arc::new(file),
            cache: Arc::new(InMemoryCache::new()),
        }
    }
}

const INPUT_VALUE: &[&[&str]] = &[
    // existing values
    &["value", "root"],
    &["value", "nested", "existing"],
    // missing values
    &["value", "missing"],
    &["value", "nested", "missing"],
];

const ARGS_VALUE: &[&[&str]] = &[
    // existing values
    &["args", "root"],
    &["args", "nested", "existing"],
    // missing values
    &["args", "missing"],
    &["args", "nested", "missing"],
];

const HEADERS_VALUE: &[&[&str]] = &[&["headers", "existing"], &["headers", "missing"]];

const VARS_VALUE: &[&[&str]] = &[&["vars", "existing"], &["vars", "missing"]];

static TEST_VALUES: Lazy<Value> = Lazy::new(|| {
    let mut root = IndexMap::new();
    let mut nested = IndexMap::new();

    nested.insert(
        Name::new("existing"),
        Value::String("nested-test".to_owned()),
    );

    root.insert(Name::new("root"), Value::String("root-test".to_owned()));
    root.insert(Name::new("nested"), Value::Object(nested));

    Value::Object(root)
});

static TEST_ARGS: Lazy<IndexMap<Name, Value>> = Lazy::new(|| {
    let mut root = IndexMap::new();
    let mut nested = IndexMap::new();

    nested.insert(
        Name::new("existing"),
        Value::String("nested-test".to_owned()),
    );

    root.insert(Name::new("root"), Value::String("root-test".to_owned()));
    root.insert(Name::new("nested"), Value::Object(nested));

    root
});

static TEST_HEADERS: Lazy<HeaderMap> = Lazy::new(|| {
    let mut map = HeaderMap::new();

    map.insert("x-existing", HeaderValue::from_static("header"));

    map
});

static TEST_VARS: Lazy<BTreeMap<String, String>> = Lazy::new(|| {
    let mut map = BTreeMap::new();

    map.insert("existing".to_owned(), "var".to_owned());

    map
});

fn to_bench_id(input: &[&str]) -> BenchmarkId {
    BenchmarkId::new("input", input.join("."))
}

struct MockGraphqlContext;

impl<'a> ResolverContextLike<'a> for MockGraphqlContext {
    fn value(&'a self) -> Option<&'a Value> {
        Some(&TEST_VALUES)
    }

    fn args(&'a self) -> Option<&'a IndexMap<Name, Value>> {
        Some(&TEST_ARGS)
    }

    fn field(&'a self) -> Option<SelectionField> {
        None
    }

    fn add_error(&'a self, _: async_graphql::ServerError) {}
}

// assert that everything was set up correctly for the benchmark
fn assert_test(eval_ctx: &EvaluationContext<'_, MockGraphqlContext>) {
    // value
    assert_eq!(
        eval_ctx.path_string(&["value", "root"]),
        Some(Cow::Borrowed("root-test"))
    );
    assert_eq!(
        eval_ctx.path_string(&["value", "nested", "existing"]),
        Some(Cow::Borrowed("nested-test"))
    );
    assert_eq!(eval_ctx.path_string(&["value", "missing"]), None);
    assert_eq!(eval_ctx.path_string(&["value", "nested", "missing"]), None);

    // args
    assert_eq!(
        eval_ctx.path_string(&["args", "root"]),
        Some(Cow::Borrowed("root-test"))
    );
    assert_eq!(
        eval_ctx.path_string(&["args", "nested", "existing"]),
        Some(Cow::Borrowed("nested-test"))
    );
    assert_eq!(eval_ctx.path_string(&["args", "missing"]), None);
    assert_eq!(eval_ctx.path_string(&["args", "nested", "missing"]), None);

    // headers
    assert_eq!(
        eval_ctx.path_string(&["headers", "x-existing"]),
        Some(Cow::Borrowed("header"))
    );
    assert_eq!(eval_ctx.path_string(&["headers", "x-missing"]), None);

    // vars
    assert_eq!(
        eval_ctx.path_string(&["vars", "existing"]),
        Some(Cow::Borrowed("var"))
    );
    assert_eq!(eval_ctx.path_string(&["vars", "missing"]), None);
}

fn request_context() -> RequestContext {
    let runtime = test::init();
    RequestContext {
        req_headers: HeaderMap::new(),
        upstream: Default::default(),
        server: Default::default(),
        http_data_loaders: Arc::new(vec![]),
        gql_data_loaders: Arc::new(vec![]),
        grpc_data_loaders: Arc::new(vec![]),
        min_max_age: Arc::new(Mutex::new(None)),
        cache_public: Arc::new(Mutex::new(None)),
        runtime,
    }
}

fn bench_main(c: &mut Criterion) {
    let mut req_ctx = request_context().req_headers(TEST_HEADERS.clone());

    req_ctx.server.vars = TEST_VARS.clone();
    let eval_ctx = EvaluationContext::new(&req_ctx, &MockGraphqlContext);

    assert_test(&eval_ctx);

    let all_inputs = INPUT_VALUE
        .iter()
        .chain(ARGS_VALUE)
        .chain(HEADERS_VALUE)
        .chain(VARS_VALUE);

    for input in all_inputs {
        c.bench_with_input(to_bench_id(input), input, |b, input| {
            b.iter(|| eval_ctx.path_string(input));
        });
    }
}

criterion_group!(benches, bench_main);
criterion_main!(benches);
