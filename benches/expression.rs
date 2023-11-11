use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use indexmap::IndexMap;
use once_cell::sync::Lazy;
use serde_json::{json, Number, Value};
use tokio::runtime::Runtime;

use tailcall::blueprint::js_plugin::JsPluginWrapper;
use tailcall::http::RequestContext;
use tailcall::lambda::{EvaluationContext, ResolverContextLike};
use tailcall::lambda::{Expression, Lambda};

use mimalloc::MiMalloc;
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

static TEST_LITERAL: Lazy<Vec<Expression>> = Lazy::new(|| {
  vec![
    Expression::Literal(Value::Number(Number::from(56))),
    Expression::Literal(Value::String(String::from("literal"))),
  ]
});

static TEST_UNSAFE_JS: Lazy<Vec<Expression>> = Lazy::new(|| {
  let js_executor = JsPluginWrapper::new("target/release").unwrap();

  vec![
    Lambda::<Value>::new(Expression::Literal(Value::Null))
      .to_unsafe_js(js_executor.clone(), "57".to_owned())
      .expression,
    Lambda::<Value>::new(Expression::Literal(Value::Null))
      .to_unsafe_js(js_executor.clone(), "'unsafe_js'".to_owned())
      .expression,
    Lambda::<Value>::new(Expression::Literal(Value::Null))
      .to_unsafe_js(
        js_executor.clone(),
        "Array(111).fill(0).reduce((acc, el, i) => acc + i, 0)".to_owned(),
      )
      .expression,
    Lambda::<Value>::new(Expression::Literal(json!("{a: 23, b: 58}")))
      .to_unsafe_js(js_executor.clone(), "ctx.a + ctx.b".to_owned())
      .expression,
  ]
});

static TESTS: &[(&str, &Lazy<Vec<Expression>>)] = &[
  ("literal", &TEST_LITERAL),
  ("unsafe-js", &TEST_UNSAFE_JS),
];

fn to_bench_id(name: &str, input: &Expression) -> BenchmarkId {
  BenchmarkId::new(name, format!("{:?}", input))
}

struct MockGraphqlContext;

impl<'a> ResolverContextLike<'a> for MockGraphqlContext {
  fn value(&'a self) -> Option<&'a async_graphql::Value> {
    None
  }

  fn args(&'a self) -> Option<&'a IndexMap<async_graphql::Name, async_graphql::Value>> {
    None
  }
}

fn bench_main(c: &mut Criterion) {
  let req_ctx = RequestContext::default();

  let eval_ctx = EvaluationContext::new(&req_ctx, &MockGraphqlContext);

  for (name, input) in TESTS {
    for input in input.iter() {
      c.bench_with_input(to_bench_id(name, input), input, |b, input| {
        b.to_async(Runtime::new().unwrap()).iter(|| input.eval(&eval_ctx))
      });
    }
  }
}

criterion_group!(benches, bench_main);
criterion_main!(benches);
