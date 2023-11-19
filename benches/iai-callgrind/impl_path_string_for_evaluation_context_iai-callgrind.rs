use std::borrow::Cow;
use std::collections::BTreeMap;

use async_graphql::{Name, Value};
use hyper::header::HeaderValue;
use hyper::HeaderMap;
use iai_callgrind::{library_benchmark, library_benchmark_group, main};
use indexmap::IndexMap;
use tailcall::http::RequestContext;
use tailcall::lambda::{EvaluationContext, ResolverContextLike};
use tailcall::path_string::PathString;

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

lazy_static::lazy_static! {
    static ref TEST_VALUES: Value = {
        let mut root = IndexMap::new();
        let mut nested = IndexMap::new();

        nested.insert(Name::new("existing"), Value::String("nested-test".to_owned()));

        root.insert(Name::new("root"), Value::String("root-test".to_owned()));
        root.insert(Name::new("nested"), Value::Object(nested));

        Value::Object(root)
    };

    static ref TEST_ARGS: IndexMap<Name, Value> = {
        let mut root = IndexMap::new();
        let mut nested = IndexMap::new();

        nested.insert(Name::new("existing"), Value::String("nested-test".to_owned()));

        root.insert(Name::new("root"), Value::String("root-test".to_owned()));
        root.insert(Name::new("nested"), Value::Object(nested));

        root
    };

    static ref TEST_HEADERS: HeaderMap = {
        let mut map = HeaderMap::new();

        map.insert("x-existing", HeaderValue::from_static("header"));

        map
    };

    static ref TEST_VARS: BTreeMap<String, String> = {
        let mut map = BTreeMap::new();

        map.insert("existing".to_owned(), "var".to_owned());

        map
    };
}

// fn to_bench_id(input: &[&str]) -> String {
//     input.join(".")
// }

struct MockGraphqlContext;

impl<'a> ResolverContextLike<'a> for MockGraphqlContext {
  fn value(&'a self) -> Option<&'a Value> {
    Some(&TEST_VALUES)
  }

  fn args(&'a self) -> Option<&'a IndexMap<Name, Value>> {
    Some(&TEST_ARGS)
  }
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

#[library_benchmark]
fn bench_main() {
  let mut req_ctx = RequestContext::default().req_headers(TEST_HEADERS.clone());

  req_ctx.server.vars = TEST_VARS.clone();

  let eval_ctx = EvaluationContext::new(&req_ctx, &MockGraphqlContext);

  assert_test(&eval_ctx);

  let all_inputs = INPUT_VALUE
    .iter()
    .chain(ARGS_VALUE)
    .chain(HEADERS_VALUE)
    .chain(VARS_VALUE);

  for input in all_inputs {
    let _result = eval_ctx.path_string(input);
  }
}

library_benchmark_group!(
    name= bench;
    benchmarks= bench_main);

main!(library_benchmark_groups = bench);
