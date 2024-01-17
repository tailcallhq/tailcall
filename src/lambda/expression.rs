use std::fmt::Debug;

use anyhow::Result;
use async_graphql_value::ConstValue;
use serde_json::Value;
use thiserror::Error;

use super::list::List;
use super::logic::Logic;
use super::{Eval, EvaluationContext, Io, Math, Relation, ResolverContextLike};
use crate::json::JsonLike;

#[derive(Clone, Debug)]
pub enum Expression {
  Context(Context),
  Literal(Value), // TODO: this should async_graphql::Value
  EqualTo(Box<Expression>, Box<Expression>),
  Io(Io),
  Input(Box<Expression>, Vec<String>),
  Logic(Logic),
  Relation(Relation),
  List(List),
  Math(Math),
  Concurrency(Concurrency, Box<Expression>),
}

#[derive(Clone, Debug)]
pub enum Concurrency {
  Parallel,
  Sequential,
}

#[derive(Clone, Debug)]
pub enum Context {
  Value,
  Path(Vec<String>),
}

#[derive(Debug, Error)]
pub enum EvaluationError {
  #[error("IOException: {0}")]
  IOException(String),

  #[error("JSException: {0}")]
  JSException(String),

  #[error("APIValidationError: {0:?}")]
  APIValidationError(Vec<String>),

  #[error("ExprEvalError: {0:?}")]
  ExprEvalError(String),
}

impl<'a> From<crate::valid::ValidationError<&'a str>> for EvaluationError {
  fn from(_value: crate::valid::ValidationError<&'a str>) -> Self {
    EvaluationError::APIValidationError(_value.as_vec().iter().map(|e| e.message.to_owned()).collect())
  }
}

impl Expression {
  pub fn concurrency(self, conc: Concurrency) -> Self {
    Expression::Concurrency(conc, Box::new(self))
  }

  pub fn in_parallel(self) -> Self {
    self.concurrency(Concurrency::Parallel)
  }

  pub fn parallel_when(self, cond: bool) -> Self {
    if cond {
      self.concurrency(Concurrency::Parallel)
    } else {
      self
    }
  }

  pub fn in_sequence(self) -> Self {
    self.concurrency(Concurrency::Sequential)
  }
}

impl Eval for Expression {
  async fn async_eval<'a, Ctx: ResolverContextLike<'a> + Sync + Send>(
    &'a self,
    ctx: &'a EvaluationContext<'a, Ctx>,
    conc: &'a Concurrency,
  ) -> Result<async_graphql::Value> {
    match self {
      Expression::Concurrency(conc, expr) => Ok(expr.eval(ctx, conc).await?),
      Expression::Context(op) => match op {
        Context::Value => Ok(ctx.value().cloned().unwrap_or(async_graphql::Value::Null)),
        Context::Path(path) => Ok(ctx.path_value(path).cloned().unwrap_or(async_graphql::Value::Null)),
      },
      Expression::Input(input, path) => {
        let inp = &input.eval(ctx, conc).await?;
        Ok(inp.get_path(path).unwrap_or(&async_graphql::Value::Null).clone())
      }
      Expression::Literal(value) => Ok(serde_json::from_value(value.clone())?),
      Expression::EqualTo(left, right) => Ok(async_graphql::Value::from(
        left.eval(ctx, conc).await? == right.eval(ctx, conc).await?,
      )),
      Expression::Io(operation) => operation.async_eval(ctx, conc).await,

      Expression::Relation(relation) => relation.async_eval(ctx, conc).await,
      Expression::Logic(logic) => logic.async_eval(ctx, conc).await,
      Expression::List(list) => list.async_eval(ctx, conc).await,
      Expression::Math(math) => math.async_eval(ctx, conc).await,
    }
  }
}

pub fn get_path_for_const_value_owned(path: &[impl AsRef<str>], mut const_value: ConstValue) -> Option<ConstValue> {
  for path in path.iter() {
    const_value = match const_value {
      ConstValue::Object(mut obj) => obj.remove(path.as_ref())?,
      _ => None?,
    }
  }

  Some(const_value)
}

pub fn get_path_for_const_value_ref<'a>(
  path: &[impl AsRef<str>],
  mut const_value: &'a ConstValue,
) -> Option<&'a ConstValue> {
  for path in path.iter() {
    const_value = match const_value {
      ConstValue::Object(ref obj) => obj.get(path.as_ref())?,
      _ => None?,
    }
  }

  Some(const_value)
}

#[cfg(test)]
mod tests {
  use async_graphql::{Name, Number, Value};
  use hyper::body::Bytes;
  use indexmap::IndexMap;

  use crate::lambda::is_truthy;

  #[test]
  fn test_is_truthy() {
    assert!(is_truthy(Value::Enum(Name::new("EXAMPLE"))));
    assert!(is_truthy(Value::List(vec![])));
    assert!(is_truthy(Value::Object(IndexMap::default())));
    assert!(is_truthy(Value::String("Hello".to_string())));
    assert!(is_truthy(Value::Boolean(true)));
    assert!(is_truthy(Value::Number(Number::from(1))));
    assert!(is_truthy(Value::Binary(Bytes::from_static(&[0, 1, 2]))));

    assert!(!is_truthy(Value::Null));
    assert!(!is_truthy(Value::String("".to_string())));
    assert!(!is_truthy(Value::Boolean(false)));
    assert!(!is_truthy(Value::Number(Number::from(0))));
    assert!(!is_truthy(Value::Binary(Bytes::default())));
  }
}
