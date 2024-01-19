use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use indexmap::IndexMap;
use mimalloc::MiMalloc;
use once_cell::sync::Lazy;
use serde_json::{Number, Value};
use tailcall::http::RequestContext;
use tailcall::lambda::{Concurrent, Eval, EvaluationContext, Expression, ResolverContextLike};
use tokio::runtime::Runtime;
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

static TEST_LITERAL: Lazy<Vec<Expression>> = Lazy::new(|| {
  vec![
    Expression::Literal(Value::Number(Number::from(56))),
    Expression::Literal(Value::String(String::from("literal"))),
  ]
});

#[cfg(feature = "unsafe-js")]
static TEST_UNSAFE_JS: Lazy<Vec<Expression>> = Lazy::new(|| {
  use serde_json::json;
  use tailcall::javascript::{JsPluginWrapper, JsPluginWrapperInterface};
  use tailcall::lambda::Lambda;

  let js_plugin = JsPluginWrapper::try_new().unwrap();

  let result = vec![
    Lambda::<Value>::new(Expression::Literal(Value::Null))
      .to_js(js_plugin.create_executor("57".to_owned(), false))
      .expression,
    Lambda::<Value>::new(Expression::Literal(Value::Null))
      .to_js(js_plugin.create_executor("'unsafe_js'".to_owned(), false))
      .expression,
    Lambda::<Value>::new(Expression::Literal(Value::Null))
      .to_js(js_plugin.create_executor(
        "Array(111).fill(0).reduce((acc, el, i) => acc + i, 0)".to_owned(),
        false,
      ))
      .expression,
    Lambda::<Value>::new(Expression::Literal(json!(
      "{ a: 'string', b: [1, 2, 3], c: { d: 546 } }"
    )))
    .to_js(js_plugin.create_executor("0.1 + 0.2".to_owned(), false))
    .expression,
    Lambda::<Value>::new(Expression::Literal(json!("{ a: 23, b: 58 }")))
      .to_js(js_plugin.create_executor("ctx.a + ctx.b".to_owned(), true))
      .expression,
  ];

  js_plugin.start().unwrap();

  result
});

static TESTS: &[(&str, &Lazy<Vec<Expression>>)] = &[
  ("literal", &TEST_LITERAL),
  #[cfg(feature = "unsafe-js")]
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

  fn field(&'a self) -> Option<async_graphql::SelectionField> {
    None
  }

  fn add_error(&'a self, _error: async_graphql::ServerError) {}
}

fn bench_main(c: &mut Criterion) {
  let req_ctx = RequestContext::default();

  let eval_ctx = EvaluationContext::new(&req_ctx, &MockGraphqlContext);

  for (name, input) in TESTS {
    for input in input.iter() {
      c.bench_with_input(to_bench_id(name, input), input, |b, input| {
        b.to_async(Runtime::new().unwrap()).iter(|| async {
          let conc = Concurrent::default();
          input.eval(&eval_ctx, &conc).await
        })
      });
    }
  }
}

criterion_group!(benches, bench_main);
criterion_main!(benches);
