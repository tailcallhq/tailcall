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
use crate::lambda::cache::Cache;

#[derive(Clone, Debug)]
pub enum Expression {
    Context(Context),
    Literal(Value), // TODO: this should async_graphql::Value
    EqualTo(Box<Expression>, Box<Expression>),
    IO(IO),
    Cache(Cache),
    Input(Box<Expression>, Vec<String>),
    Logic(Logic),
    Relation(Relation),
    List(List),
    Math(Math),
    Concurrency(Concurrent, Box<Expression>),
    Protected(Box<Expression>),
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

    #[error("APIValidationError: {0:?}")]
    APIValidationError(Vec<String>),

    #[error("ExprEvalError: {0:?}")]
    ExprEvalError(String),
}

impl<'a> From<crate::valid::ValidationError<&'a str>> for EvaluationError {
    fn from(_value: crate::valid::ValidationError<&'a str>) -> Self {
        EvaluationError::APIValidationError(
            _value
                .as_vec()
                .iter()
                .map(|e| e.message.to_owned())
                .collect(),
        )
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
                    Context::Value => {
                        Ok(ctx.value().cloned().unwrap_or(async_graphql::Value::Null))
                    }
                    Context::Path(path) => Ok(ctx
                        .path_value(path)
                        .cloned()
                        .unwrap_or(async_graphql::Value::Null)),
                },
                Expression::Input(input, path) => {
                    let inp = &input.eval(ctx, conc).await?;
                    Ok(inp
                        .get_path(path)
                        .unwrap_or(&async_graphql::Value::Null)
                        .clone())
                }
                Expression::Literal(value) => Ok(serde_json::from_value(value.clone())?),
                Expression::EqualTo(left, right) => Ok(async_graphql::Value::from(
                    left.eval(ctx, conc).await? == right.eval(ctx, conc).await?,
                )),
                Expression::Protected(expr) => {
                    ctx.req_ctx.auth_ctx.validate(ctx.req_ctx).await?;
                    expr.eval(ctx, conc).await
                }
                Expression::IO(operation) => operation.eval(ctx, conc).await,
                Expression::Cache(cached) => cached.eval(ctx, conc).await,
                Expression::Relation(relation) => relation.eval(ctx, conc).await,
                Expression::Logic(logic) => logic.eval(ctx, conc).await,
                Expression::List(list) => list.eval(ctx, conc).await,
                Expression::Math(math) => math.eval(ctx, conc).await,
            }
        })
    }
}
