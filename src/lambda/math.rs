use core::future::Future;
use std::ops;
use std::pin::Pin;

use anyhow::Result;
use async_graphql_value::ConstValue;

use super::{Concurrent, Eval, EvaluationContext, EvaluationError, Expression, ResolverContextLike};
use crate::json::JsonLike;

#[derive(Clone, Debug)]
pub enum Math {
  Mod(Box<Expression>, Box<Expression>),
  Add(Box<Expression>, Box<Expression>),
  Dec(Box<Expression>),
  Divide(Box<Expression>, Box<Expression>),
  Inc(Box<Expression>),
  Multiply(Box<Expression>, Box<Expression>),
  Negate(Box<Expression>),
  Product(Vec<Expression>),
  Subtract(Box<Expression>, Box<Expression>),
  Sum(Vec<Expression>),
  Mean(Vec<Expression>),
  Median(Vec<Expression>),
}

impl Eval for Math {
  fn eval<'a, Ctx: ResolverContextLike<'a> + Sync + Send>(
    &'a self,
    ctx: &'a EvaluationContext<'a, Ctx>,
    conc: &'a Concurrent,
  ) -> Pin<Box<dyn Future<Output = Result<ConstValue>> + 'a + Send>> {
    Box::pin(async move {
      Ok(match self {
        Math::Mod(lhs, rhs) => {
          let lhs = lhs.eval(ctx, conc).await?;
          let rhs = rhs.eval(ctx, conc).await?;

          try_i64_operation(&lhs, &rhs, ops::Rem::rem)
            .or_else(|| try_u64_operation(&lhs, &rhs, ops::Rem::rem))
            .ok_or(EvaluationError::ExprEvalError("mod".into()))?
        }
        Math::Add(lhs, rhs) => {
          let lhs = lhs.eval(ctx, conc).await?;
          let rhs = rhs.eval(ctx, conc).await?;

          try_f64_operation(&lhs, &rhs, ops::Add::add)
            .or_else(|| try_u64_operation(&lhs, &rhs, ops::Add::add))
            .or_else(|| try_i64_operation(&lhs, &rhs, ops::Add::add))
            .ok_or(EvaluationError::ExprEvalError("add".into()))?
        }
        Math::Dec(val) => {
          let val = val.eval(ctx, conc).await?;

          val
            .as_f64_ok()
            .ok()
            .map(|val| (val - 1f64).into())
            .or_else(|| val.as_u64_ok().ok().map(|val| (val - 1u64).into()))
            .or_else(|| val.as_i64_ok().ok().map(|val| (val - 1i64).into()))
            .ok_or(EvaluationError::ExprEvalError("dec".into()))?
        }
        Math::Divide(lhs, rhs) => {
          let lhs = lhs.eval(ctx, conc).await?;
          let rhs = rhs.eval(ctx, conc).await?;

          try_div_operation(&lhs, &rhs, None)?
        }
        Math::Median(exprs) => {
          let mut results: Vec<_> = exprs.eval(ctx, conc).await?;

          let len = results.len();

          let width = 2 - len % 2;
          let idx = (len - width) / 2;

          results.sort_by(|a, b| {
            let a = a.as_f64_ok().unwrap_or(0f64);
            let b = b.as_f64_ok().unwrap_or(0f64);

            a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal)
          });

          let slice = results
            .get(idx..idx + width)
            .ok_or(EvaluationError::ExprEvalError("median".into()))?;

          let value = try_mean_operation(slice, Some("median"))?;

          let as_f64 = value.as_f64_ok().unwrap_or(0f64);

          if as_f64 == 0f64 {
            return Err(EvaluationError::ExprEvalError("median can not be zero".into()).into());
          } else if as_f64 < 0f64 {
            return Err(EvaluationError::ExprEvalError("median can not be negative".into()).into());
          }

          value
        }
        Math::Inc(val) => {
          let val = val.eval(ctx, conc).await?;

          val
            .as_f64_ok()
            .ok()
            .map(|val| (val + 1f64).into())
            .or_else(|| val.as_u64_ok().ok().map(|val| (val + 1u64).into()))
            .or_else(|| val.as_i64_ok().ok().map(|val| (val + 1i64).into()))
            .ok_or(EvaluationError::ExprEvalError("dec".into()))?
        }
        Math::Multiply(lhs, rhs) => {
          let lhs = lhs.eval(ctx, conc).await?;
          let rhs = rhs.eval(ctx, conc).await?;

          try_f64_operation(&lhs, &rhs, ops::Mul::mul)
            .or_else(|| try_u64_operation(&lhs, &rhs, ops::Mul::mul))
            .or_else(|| try_i64_operation(&lhs, &rhs, ops::Mul::mul))
            .ok_or(EvaluationError::ExprEvalError("multiply".into()))?
        }
        Math::Negate(val) => {
          let val = val.eval(ctx, conc).await?;

          val
            .as_f64_ok()
            .ok()
            .map(|val| (-val).into())
            .or_else(|| val.as_i64_ok().ok().map(|val| (-val).into()))
            .ok_or(EvaluationError::ExprEvalError("neg".into()))?
        }
        Math::Product(exprs) => {
          let results: Vec<_> = exprs.eval(ctx, conc).await?;

          results.into_iter().try_fold(1i64.into(), |lhs, rhs| {
            try_f64_operation(&lhs, &rhs, ops::Mul::mul)
              .or_else(|| try_u64_operation(&lhs, &rhs, ops::Mul::mul))
              .or_else(|| try_i64_operation(&lhs, &rhs, ops::Mul::mul))
              .ok_or(EvaluationError::ExprEvalError("product".into()))
          })?
        }
        Math::Mean(exprs) => {
          let results: Vec<_> = exprs.eval(ctx, conc).await?;

          try_mean_operation(&results, None)?
        }
        Math::Subtract(lhs, rhs) => {
          let lhs = lhs.eval(ctx, conc).await?;
          let rhs = rhs.eval(ctx, conc).await?;

          try_f64_operation(&lhs, &rhs, ops::Sub::sub)
            .or_else(|| try_u64_operation(&lhs, &rhs, ops::Sub::sub))
            .or_else(|| try_i64_operation(&lhs, &rhs, ops::Sub::sub))
            .ok_or(EvaluationError::ExprEvalError("subtract".into()))?
        }
        Math::Sum(exprs) => {
          let results: Vec<_> = exprs.eval(ctx, conc).await?;

          try_sum_operation(&results, None)?
        }
      })
    })
  }
}

fn format_error(error: Option<&str>, op: &str) -> String {
  error.map(|e| format!("{}-{}", e, op)).unwrap_or(op.into())
}

fn try_mean_operation(exprs: &[ConstValue], error: Option<&str>) -> Result<ConstValue, EvaluationError> {
  let error = format_error(error, "mean");

  try_sum_operation(exprs, Some(&error))
    .ok()
    .map(|sum| try_div_operation(&sum, &exprs.len().into(), Some(&error)))
    .ok_or(EvaluationError::ExprEvalError(error))?
}

fn try_sum_operation(exprs: &[ConstValue], error: Option<&str>) -> Result<ConstValue, EvaluationError> {
  exprs.iter().try_fold(0i64.into(), |lhs, rhs| {
    try_f64_operation(&lhs, rhs, ops::Add::add)
      .or_else(|| try_u64_operation(&lhs, rhs, ops::Add::add))
      .or_else(|| try_i64_operation(&lhs, rhs, ops::Add::add))
      .ok_or(EvaluationError::ExprEvalError(format_error(error, "sum")))
  })
}

fn try_div_operation(lhs: &ConstValue, rhs: &ConstValue, error: Option<&str>) -> Result<ConstValue, EvaluationError> {
  try_f64_operation(lhs, rhs, ops::Div::div)
    .or_else(|| try_u64_operation(lhs, rhs, ops::Div::div))
    .or_else(|| try_i64_operation(lhs, rhs, ops::Div::div))
    .ok_or(EvaluationError::ExprEvalError(format_error(error, "div")))
}

fn try_f64_operation<F>(lhs: &ConstValue, rhs: &ConstValue, f: F) -> Option<ConstValue>
where
  F: Fn(f64, f64) -> f64,
{
  match (lhs, rhs) {
    (ConstValue::Number(lhs), ConstValue::Number(rhs)) => {
      lhs.as_f64().and_then(|lhs| rhs.as_f64().map(|rhs| f(lhs, rhs).into()))
    }
    _ => None,
  }
}

fn try_i64_operation<F>(lhs: &ConstValue, rhs: &ConstValue, f: F) -> Option<ConstValue>
where
  F: Fn(i64, i64) -> i64,
{
  match (lhs, rhs) {
    (ConstValue::Number(lhs), ConstValue::Number(rhs)) => {
      lhs.as_i64().and_then(|lhs| rhs.as_i64().map(|rhs| f(lhs, rhs).into()))
    }
    _ => None,
  }
}

fn try_u64_operation<F>(lhs: &ConstValue, rhs: &ConstValue, f: F) -> Option<ConstValue>
where
  F: Fn(u64, u64) -> u64,
{
  match (lhs, rhs) {
    (ConstValue::Number(lhs), ConstValue::Number(rhs)) => {
      lhs.as_u64().and_then(|lhs| rhs.as_u64().map(|rhs| f(lhs, rhs).into()))
    }
    _ => None,
  }
}
