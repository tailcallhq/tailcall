use std::borrow::Cow;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use async_graphql::context::SelectionField;
use async_graphql::{Name, Value};
use async_trait::async_trait;
use criterion::{BenchmarkId, Criterion};
use http_cache_reqwest::{Cache, CacheMode, HttpCache, HttpCacheOptions};
use hyper::body::Bytes;
use hyper::header::HeaderValue;
use hyper::HeaderMap;
use indexmap::IndexMap;
use once_cell::sync::Lazy;
use reqwest::{Client, Request};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use tailcall::core::blueprint::{Server, Upstream};
use tailcall::core::cache::InMemoryCache;
use tailcall::core::http::{RequestContext, Response};
use tailcall::core::lambda::{EvaluationContext, ResolverContextLike};
use tailcall::core::path::PathString;
use tailcall::core::runtime::TargetRuntime;
use tailcall::core::{EnvIO, FileIO, HttpIO};
use tailcall_http_cache::HttpCacheManager;

struct Http {
    client: ClientWithMiddleware,
    http2_only: bool,
}

impl Http {
    fn init(upstream: &Upstream) -> Self {
        let mut builder = Client::builder()
            .tcp_keepalive(Some(Duration::from_secs(upstream.tcp_keep_alive)))
            .timeout(Duration::from_secs(upstream.timeout))
            .connect_timeout(Duration::from_secs(upstream.connect_timeout))
            .http2_keep_alive_interval(Some(Duration::from_secs(upstream.keep_alive_interval)))
            .http2_keep_alive_timeout(Duration::from_secs(upstream.keep_alive_timeout))
            .http2_keep_alive_while_idle(upstream.keep_alive_while_idle)
            .pool_idle_timeout(Some(Duration::from_secs(upstream.pool_idle_timeout)))
            .pool_max_idle_per_host(upstream.pool_max_idle_per_host)
            .user_agent(upstream.user_agent.clone());

        // Add Http2 Prior Knowledge
        if upstream.http2_only {
            builder = builder.http2_prior_knowledge();
        }

        // Add Http Proxy
        if let Some(ref proxy) = upstream.proxy {
            builder = builder.proxy(
                reqwest::Proxy::http(proxy.url.clone())
                    .expect("Failed to set proxy in http client"),
            );
        }

        let mut client = ClientBuilder::new(builder.build().expect("Failed to build client"));

        if upstream.http_cache {
            client = client.with(Cache(HttpCache {
                mode: CacheMode::Default,
                manager: HttpCacheManager::default(),
                options: HttpCacheOptions::default(),
            }))
        }
        Self { client: client.build(), http2_only: upstream.http2_only }
    }
}

#[async_trait]
impl HttpIO for Http {
    async fn execute(&self, mut request: Request) -> anyhow::Result<Response<Bytes>> {
        if self.http2_only {
            *request.version_mut() = reqwest::Version::HTTP_2;
        }
        let resp = self.client.execute(request).await?;
        Response::from_reqwest(resp).await
    }
}

struct Env {}

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

#[derive(Clone)]
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
    let config_module = tailcall::core::config::ConfigModule::default();

    //TODO: default is used only in tests. Drop default and move it to test.
    let upstream = Upstream::try_from(&config_module).unwrap();
    let server = Server::try_from(config_module).unwrap();
    let http = Arc::new(Http::init(&upstream));
    let http2 = Arc::new(Http::init(&upstream.clone().http2_only(true)));
    let runtime = TargetRuntime {
        http2_only: http2,
        http,
        env: Arc::new(Env {}),
        file: Arc::new(File {}),
        cache: Arc::new(InMemoryCache::new()),
        extensions: Arc::new(vec![]),
    };
    RequestContext::new(runtime)
        .server(server)
        .upstream(upstream)
}

pub fn bench_main(c: &mut Criterion) {
    let mut req_ctx = request_context().allowed_headers(TEST_HEADERS.clone());

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
