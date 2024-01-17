use std::collections::HashSet;

use anyhow::Result;
use async_graphql_value::ConstValue;
use futures_util::future::join_all;

use super::{Concurrency, Eval, EvaluationContext, Expression, ResolverContextLike};
use crate::helpers::value::HashableConstValue;

/// Check if a value is truthy
///
/// Special cases:
/// 1. An empty string is considered falsy
/// 2. A collection of bytes is truthy, even if the value in those bytes is 0. An empty collection is falsy.
pub fn is_truthy(value: async_graphql::Value) -> bool {
  use async_graphql::{Number, Value};
  use hyper::body::Bytes;

  match value {
    Value::Null => false,
    Value::Enum(_) => true,
    Value::List(_) => true,
    Value::Object(_) => true,
    Value::String(s) => !s.is_empty(),
    Value::Boolean(b) => b,
    Value::Number(n) => n != Number::from(0),
    Value::Binary(b) => b != Bytes::default(),
  }
}

pub fn is_truthy_ref(value: &async_graphql::Value) -> bool {
  use async_graphql::{Number, Value};
  use hyper::body::Bytes;

  match value {
    &Value::Null => false,
    &Value::Enum(_) => true,
    &Value::List(_) => true,
    &Value::Object(_) => true,
    Value::String(s) => !s.is_empty(),
    &Value::Boolean(b) => b,
    Value::Number(n) => n != &Number::from(0),
    Value::Binary(b) => b != &Bytes::default(),
  }
}

pub fn is_empty(value: &async_graphql::Value) -> bool {
  match value {
    ConstValue::Null => true,
    ConstValue::Number(_) | ConstValue::Boolean(_) | ConstValue::Enum(_) => false,
    ConstValue::Binary(bytes) => bytes.is_empty(),
    ConstValue::List(list) => list.is_empty(),
    ConstValue::Object(obj) => obj.is_empty(),
    ConstValue::String(string) => string.is_empty(),
  }
}

#[allow(clippy::too_many_arguments)]
pub async fn set_operation<'a, 'b, Ctx: ResolverContextLike<'a> + Sync + Send, F>(
  ctx: &'a EvaluationContext<'a, Ctx>,
  conc: &'a Concurrency,
  lhs: &'a [Expression],
  rhs: &'a [Expression],
  operation: F,
) -> Result<ConstValue>
where
  F: Fn(HashSet<HashableConstValue>, HashSet<HashableConstValue>) -> Vec<ConstValue>,
{
  let lhs = eval_map_list_expressions(ctx, conc, lhs, HashableConstValue).await?;
  let rhs = eval_map_list_expressions(ctx, conc, rhs, HashableConstValue).await?;
  Ok(operation(lhs, rhs).into())
}

#[allow(clippy::redundant_closure, clippy::too_many_arguments)]
pub async fn eval_map_list_expressions<
  'a,
  Ctx: ResolverContextLike<'a> + Sync + Send,
  O,
  C: FromIterator<O>,
  F: Fn(ConstValue) -> O,
>(
  ctx: &'a EvaluationContext<'a, Ctx>,
  conc: &'a Concurrency,
  exprs: &'a [Expression],
  f: F,
) -> Result<C> {
  let future_iter = exprs.iter().map(|expr| expr.eval(ctx, conc));
  match *conc {
    Concurrency::Parallel => join_all(future_iter)
      .await
      .into_iter()
      .map(|result| result.map(|cv| f(cv)))
      .collect::<Result<C>>(),
    Concurrency::Sequential => {
      let mut results = Vec::with_capacity(exprs.len());
      for future in future_iter {
        results.push(f(future.await?));
      }
      Ok(results.into_iter().collect())
    }
  }
}
