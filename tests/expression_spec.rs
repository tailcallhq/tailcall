use async_graphql::Value;
use pretty_assertions::assert_eq;
use serde_json::json;

use tailcall::{
    blueprint::{DynamicValue, Upstream},
    http::RequestContext,
    lambda::{Concurrent, EmptyResolverContext, Eval, EvaluationContext, Expression},
    mustache::Mustache,
};

async fn eval(expr: &Expression) -> anyhow::Result<Value> {
    let runtime = tailcall::cli::runtime::init(&Upstream::default(), None);
    let req_ctx = RequestContext::new(runtime);
    let res_ctx = EmptyResolverContext {};
    let eval_ctx = EvaluationContext::new(&req_ctx, &res_ctx);
    expr.eval(eval_ctx, &Concurrent::Parallel).await
}

#[tokio::test]
async fn test_no_key() {
    let abcde = DynamicValue::try_from(&json!({"a": {"b": {"c": {"d": "e"}}}})).unwrap();
    let expr = Expression::Literal(abcde)
        .and_then(Expression::Literal(DynamicValue::Mustache(
            Mustache::parse("{{value.a}}").unwrap(),
        )))
        .and_then(Expression::Literal(DynamicValue::Mustache(
            Mustache::parse("{{value.b}}").unwrap(),
        )))
        .and_then(Expression::Literal(DynamicValue::Mustache(
            Mustache::parse("{{value.c}}").unwrap(),
        )))
        .and_then(Expression::Literal(DynamicValue::Mustache(
            Mustache::parse("{{value.d}}").unwrap(),
        )));

    let actual = eval(&expr).await.unwrap();
    let expected = Value::from_json(json!("e")).unwrap();

    assert_eq!(actual, expected);
}
