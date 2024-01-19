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
use tailcall::blueprint::Server;
use tailcall::cli::chrono_cache::NativeChronoCache;
use tailcall::cli::{init_env, init_http, init_http2_only};
use tailcall::http::RequestContext;
use tailcall::lambda::{EvaluationContext, ResolverContextLike};
use tailcall::path::PathString;

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

  nested.insert(Name::new("existing"), Value::String("nested-test".to_owned()));

  root.insert(Name::new("root"), Value::String("root-test".to_owned()));
  root.insert(Name::new("nested"), Value::Object(nested));

  Value::Object(root)
});

static TEST_ARGS: Lazy<IndexMap<Name, Value>> = Lazy::new(|| {
  let mut root = IndexMap::new();
  let mut nested = IndexMap::new();

  nested.insert(Name::new("existing"), Value::String("nested-test".to_owned()));

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
  assert_eq!(eval_ctx.path_string(&["vars", "existing"]), Some(Cow::Borrowed("var")));
  assert_eq!(eval_ctx.path_string(&["vars", "missing"]), None);
}

fn request_context() -> RequestContext {
  let tailcall::config::Config { server, upstream, .. } = tailcall::config::Config::default();
  //TODO: default is used only in tests. Drop default and move it to test.
  let server = Server::try_from(server).unwrap();

  let h_client = Arc::new(init_http(&upstream));
  let h2_client = Arc::new(init_http2_only(&upstream));
  RequestContext {
    req_headers: HeaderMap::new(),
    h_client,
    h2_client,
    server,
    upstream,
    http_data_loaders: Arc::new(vec![]),
    gql_data_loaders: Arc::new(vec![]),
    cache: Arc::new(NativeChronoCache::new()),
    grpc_data_loaders: Arc::new(vec![]),
    min_max_age: Arc::new(Mutex::new(None)),
    cache_public: Arc::new(Mutex::new(None)),
    env_vars: Arc::new(init_env()),
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
