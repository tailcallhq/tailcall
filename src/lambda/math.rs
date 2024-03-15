use core::future::Future;
use std::ops;
use std::pin::Pin;

use anyhow::Result;
use async_graphql_value::ConstValue;

use super::{
    Concurrent, Eval, EvaluationContext, EvaluationError, Expression, ResolverContextLike,
};
use crate::json::JsonLike;

#[derive(Clone, Debug, strum_macros::Display)]
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

                    val.as_f64_ok()
                        .ok()
                        .map(|val| (val - 1f64).into())
                        .or_else(|| val.as_u64_ok().ok().map(|val| (val - 1u64).into()))
                        .or_else(|| val.as_i64_ok().ok().map(|val| (val - 1i64).into()))
                        .ok_or(EvaluationError::ExprEvalError("dec".into()))?
                }
                Math::Divide(lhs, rhs) => {
                    let lhs = lhs.eval(ctx, conc).await?;
                    let rhs = rhs.eval(ctx, conc).await?;

                    try_f64_operation(&lhs, &rhs, ops::Div::div)
                        .or_else(|| try_u64_operation(&lhs, &rhs, ops::Div::div))
                        .or_else(|| try_i64_operation(&lhs, &rhs, ops::Div::div))
                        .ok_or(EvaluationError::ExprEvalError("divide".into()))?
                }
                Math::Inc(val) => {
                    let val = val.eval(ctx, conc).await?;

                    val.as_f64_ok()
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

                    val.as_f64_ok()
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

                    results.into_iter().try_fold(0i64.into(), |lhs, rhs| {
                        try_f64_operation(&lhs, &rhs, ops::Add::add)
                            .or_else(|| try_u64_operation(&lhs, &rhs, ops::Add::add))
                            .or_else(|| try_i64_operation(&lhs, &rhs, ops::Add::add))
                            .ok_or(EvaluationError::ExprEvalError("sum".into()))
                    })?
                }
            })
        })
    }
}

fn try_f64_operation<F>(lhs: &ConstValue, rhs: &ConstValue, f: F) -> Option<ConstValue>
where
    F: Fn(f64, f64) -> f64,
{
    match (lhs, rhs) {
        (ConstValue::Number(lhs), ConstValue::Number(rhs)) => lhs
            .as_f64()
            .and_then(|lhs| rhs.as_f64().map(|rhs| f(lhs, rhs).into())),
        _ => None,
    }
}

fn try_i64_operation<F>(lhs: &ConstValue, rhs: &ConstValue, f: F) -> Option<ConstValue>
where
    F: Fn(i64, i64) -> i64,
{
    match (lhs, rhs) {
        (ConstValue::Number(lhs), ConstValue::Number(rhs)) => lhs
            .as_i64()
            .and_then(|lhs| rhs.as_i64().map(|rhs| f(lhs, rhs).into())),
        _ => None,
    }
}

fn try_u64_operation<F>(lhs: &ConstValue, rhs: &ConstValue, f: F) -> Option<ConstValue>
where
    F: Fn(u64, u64) -> u64,
{
    match (lhs, rhs) {
        (ConstValue::Number(lhs), ConstValue::Number(rhs)) => lhs
            .as_u64()
            .and_then(|lhs| rhs.as_u64().map(|rhs| f(lhs, rhs).into())),
        _ => None,
    }
}
