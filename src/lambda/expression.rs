use core::future::Future;
use std::fmt::Debug;
use std::pin::Pin;

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
  fn eval<'a, Ctx: ResolverContextLike<'a> + Sync + Send>(
    &'a self,
    ctx: &'a EvaluationContext<'a, Ctx>,
    conc: &'a Concurrency,
  ) -> Pin<Box<dyn Future<Output = Result<ConstValue>> + 'a + Send>> {
    Box::pin(async move {
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
        Expression::Io(operation) => operation.eval(ctx, conc).await,

        Expression::Relation(relation) => relation.eval(ctx, conc).await,
        Expression::Logic(logic) => logic.eval(ctx, conc).await,
        Expression::List(list) => list.eval(ctx, conc).await,
        Expression::Math(math) => math.eval(ctx, conc).await,
      }
    })
  }
}
