use std::ops;

use async_graphql_value::ConstValue;

use super::{Concurrency, Eval, EvaluationContext, EvaluationError, Expression, ResolverContextLike};
use crate::helpers::value::{try_f64_operation, try_i64_operation, try_u64_operation};
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
}

impl Eval for Math {
  async fn async_eval<'a, Ctx: ResolverContextLike<'a> + Sync + Send>(
    &'a self,
    ctx: &'a EvaluationContext<'a, Ctx>,
    conc: &'a Concurrency,
  ) -> anyhow::Result<ConstValue> {
    Ok(match self {
      Math::Mod(lhs, rhs) => {
        let lhs = lhs.eval(ctx, conc).await?;
        let rhs = rhs.eval(ctx, conc).await?;

        try_i64_operation(&lhs, &rhs, ops::Rem::rem)
          .or_else(|| try_u64_operation(&lhs, &rhs, ops::Rem::rem))
          .ok_or(EvaluationError::OperationFailed("mod".into()))?
      }
      Math::Add(lhs, rhs) => {
        let lhs = lhs.eval(ctx, conc).await?;
        let rhs = rhs.eval(ctx, conc).await?;

        try_f64_operation(&lhs, &rhs, ops::Add::add)
          .or_else(|| try_u64_operation(&lhs, &rhs, ops::Add::add))
          .or_else(|| try_i64_operation(&lhs, &rhs, ops::Add::add))
          .ok_or(EvaluationError::OperationFailed("add".into()))?
      }
      Math::Dec(val) => {
        let val = val.eval(ctx, conc).await?;

        val
          .as_f64_ok()
          .ok()
          .map(|val| (val - 1f64).into())
          .or_else(|| val.as_u64_ok().ok().map(|val| (val - 1u64).into()))
          .or_else(|| val.as_i64_ok().ok().map(|val| (val - 1i64).into()))
          .ok_or(EvaluationError::OperationFailed("dec".into()))?
      }
      Math::Divide(lhs, rhs) => {
        let lhs = lhs.eval(ctx, conc).await?;
        let rhs = rhs.eval(ctx, conc).await?;

        try_f64_operation(&lhs, &rhs, ops::Div::div)
          .or_else(|| try_u64_operation(&lhs, &rhs, ops::Div::div))
          .or_else(|| try_i64_operation(&lhs, &rhs, ops::Div::div))
          .ok_or(EvaluationError::OperationFailed("divide".into()))?
      }
      Math::Inc(val) => {
        let val = val.eval(ctx, conc).await?;

        val
          .as_f64_ok()
          .ok()
          .map(|val| (val + 1f64).into())
          .or_else(|| val.as_u64_ok().ok().map(|val| (val + 1u64).into()))
          .or_else(|| val.as_i64_ok().ok().map(|val| (val + 1i64).into()))
          .ok_or(EvaluationError::OperationFailed("dec".into()))?
      }
      Math::Multiply(lhs, rhs) => {
        let lhs = lhs.eval(ctx, conc).await?;
        let rhs = rhs.eval(ctx, conc).await?;

        try_f64_operation(&lhs, &rhs, ops::Mul::mul)
          .or_else(|| try_u64_operation(&lhs, &rhs, ops::Mul::mul))
          .or_else(|| try_i64_operation(&lhs, &rhs, ops::Mul::mul))
          .ok_or(EvaluationError::OperationFailed("multiply".into()))?
      }
      Math::Negate(val) => {
        let val = val.eval(ctx, conc).await?;

        val
          .as_f64_ok()
          .ok()
          .map(|val| (-val).into())
          .or_else(|| val.as_i64_ok().ok().map(|val| (-val).into()))
          .ok_or(EvaluationError::OperationFailed("neg".into()))?
      }
      Math::Product(exprs) => {
        let results: Vec<_> = exprs.eval(ctx, conc).await?;

        results.into_iter().try_fold(1i64.into(), |lhs, rhs| {
          try_f64_operation(&lhs, &rhs, ops::Mul::mul)
            .or_else(|| try_u64_operation(&lhs, &rhs, ops::Mul::mul))
            .or_else(|| try_i64_operation(&lhs, &rhs, ops::Mul::mul))
            .ok_or(EvaluationError::OperationFailed("product".into()))
        })?
      }
      Math::Subtract(lhs, rhs) => {
        let lhs = lhs.eval(ctx, conc).await?;
        let rhs = rhs.eval(ctx, conc).await?;

        try_f64_operation(&lhs, &rhs, ops::Sub::sub)
          .or_else(|| try_u64_operation(&lhs, &rhs, ops::Sub::sub))
          .or_else(|| try_i64_operation(&lhs, &rhs, ops::Sub::sub))
          .ok_or(EvaluationError::OperationFailed("subtract".into()))?
      }
      Math::Sum(exprs) => {
        let results: Vec<_> = exprs.eval(ctx, conc).await?;

        results.into_iter().try_fold(0i64.into(), |lhs, rhs| {
          try_f64_operation(&lhs, &rhs, ops::Add::add)
            .or_else(|| try_u64_operation(&lhs, &rhs, ops::Add::add))
            .or_else(|| try_i64_operation(&lhs, &rhs, ops::Add::add))
            .ok_or(EvaluationError::OperationFailed("sum".into()))
        })?
      }
    })
  }
}
