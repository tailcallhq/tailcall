#![deny(clippy::wildcard_enum_match_arm)] // to make sure we handle all of the expression variants

use core::future::Future;
use std::fmt::Debug;
use std::pin::Pin;

use anyhow::Result;
use async_graphql_value::ConstValue;
use serde_json::Value;
use thiserror::Error;

use super::list::List;
use super::logic::Logic;
use super::{Concurrent, Eval, EvaluationContext, Math, Relation, ResolverContextLike, IO};
use crate::json::JsonLike;

#[derive(Clone, Debug)]
pub enum Expression {
  Context(Context),
  Literal(Value), // TODO: this should async_graphql::Value
  EqualTo(Box<Expression>, Box<Expression>),
  IO(IO),
  Input(Box<Expression>, Vec<String>),
  Logic(Logic),
  Relation(Relation),
  List(List),
  Math(Math),
  Concurrency(Concurrent, Box<Expression>),
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
  pub fn concurrency(self, conc: Concurrent) -> Self {
    Expression::Concurrency(conc, Box::new(self))
  }

  pub fn in_parallel(self) -> Self {
    self.concurrency(Concurrent::Parallel)
  }

  pub fn parallel_when(self, cond: bool) -> Self {
    if cond {
      self.concurrency(Concurrent::Parallel)
    } else {
      self
    }
  }

  pub fn in_sequence(self) -> Self {
    self.concurrency(Concurrent::Sequential)
  }
}

impl Eval for Expression {
  fn eval<'a, Ctx: ResolverContextLike<'a> + Sync + Send>(
    &'a self,
    ctx: &'a EvaluationContext<'a, Ctx>,
    conc: &'a Concurrent,
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
        Expression::IO(operation) => operation.eval(ctx, conc).await,
        Expression::Relation(relation) => relation.eval(ctx, conc).await,
        Expression::Logic(logic) => logic.eval(ctx, conc).await,
        Expression::List(list) => list.eval(ctx, conc).await,
        Expression::Math(math) => math.eval(ctx, conc).await,
      }
    })
  }
}

impl Expression {
  pub(crate) fn for_each_mut<F: FnMut(&mut Expression)>(&mut self, f: &mut F) {
    f(self);

    match self {
      Expression::Context(_) | Expression::Literal(_) | Expression::IO(_) => {}
      Expression::EqualTo(left, right) => {
        left.for_each_mut(f);
        right.for_each_mut(f);
      }
      Expression::Input(expr, _) | Expression::Concurrency(_, expr) => expr.for_each_mut(f),
      Expression::Logic(logic) => match logic {
        Logic::If { cond, then, els } => {
          cond.for_each_mut(f);
          then.for_each_mut(f);
          els.for_each_mut(f);
        }
        Logic::And(exprs) | Logic::Or(exprs) => {
          for expr in exprs {
            expr.for_each_mut(f)
          }
        }
        Logic::Cond(exprs) => exprs.iter_mut().for_each(|(left, right)| {
          left.for_each_mut(f);
          right.for_each_mut(f);
        }),
        Logic::DefaultTo(left, right) => {
          left.for_each_mut(f);
          right.for_each_mut(f);
        }
        Logic::IsEmpty(expr) | Logic::Not(expr) => {
          expr.for_each_mut(f);
        }
      },
      Expression::Relation(relation) => match relation {
        Relation::Intersection(exprs) | Relation::Max(exprs) | Relation::Min(exprs) => {
          for expr in exprs {
            expr.for_each_mut(f);
          }
        }
        Relation::Equals(left, right)
        | Relation::Gt(left, right)
        | Relation::Gte(left, right)
        | Relation::Lt(left, right)
        | Relation::Lte(left, right)
        | Relation::PathEq(left, _, right)
        | Relation::PropEq(left, _, right) => {
          left.for_each_mut(f);
          right.for_each_mut(f);
        }
        Relation::Difference(exprs_left, exprs_right)
        | Relation::SymmetricDifference(exprs_left, exprs_right)
        | Relation::Union(exprs_left, exprs_right) => {
          for expr in exprs_left.iter_mut().chain(exprs_right.iter_mut()) {
            expr.for_each_mut(f);
          }
        }
        Relation::SortPath(expr, _) => {
          expr.for_each_mut(f);
        }
      },
      Expression::List(list) => match list {
        List::Concat(exprs) => {
          for expr in exprs {
            expr.for_each_mut(f)
          }
        }
      },
      Expression::Math(math) => match math {
        Math::Inc(expr) | Math::Negate(expr) | Math::Dec(expr) => {
          expr.for_each_mut(f);
        }
        Math::Mod(left, right)
        | Math::Add(left, right)
        | Math::Divide(left, right)
        | Math::Multiply(left, right)
        | Math::Subtract(left, right) => {
          left.for_each_mut(f);
          right.for_each_mut(f);
        }
        Math::Sum(exprs) | Math::Product(exprs) => {
          for expr in exprs {
            expr.for_each_mut(f);
          }
        }
      },
    }
  }
}
