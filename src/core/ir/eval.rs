use std::future::Future;
use std::ops::Deref;

use async_graphql_value::ConstValue;

use super::model::{Cache, CacheKey, Context, Map, IR};
use super::{EvaluationContext, EvaluationError, ResolverContextLike};
use crate::core::json::JsonLike;
use crate::core::serde_value_ext::ValueExt;

pub trait Eval<Output = async_graphql::Value> {
    fn eval<Ctx>(
        &self,
        ctx: &mut EvaluationContext<'_, Ctx>,
    ) -> impl Future<Output = Result<Output, EvaluationError>>
    where
        Ctx: ResolverContextLike + Sync;
}

impl Eval for IR {
    #[tracing::instrument(skip_all, fields(otel.name = %self), err)]
    fn eval<Ctx>(
        &self,
        ctx: &mut EvaluationContext<'_, Ctx>,
    ) -> impl Future<Output = Result<ConstValue, EvaluationError>>
    where
        Ctx: ResolverContextLike + Sync,
    {
        Box::pin(async move {
            match self {
                IR::Context(op) => match op {
                    Context::Value => {
                        Ok(ctx.value().cloned().unwrap_or(async_graphql::Value::Null))
                    }
                    Context::Path(path) => Ok(ctx
                        .path_value(path)
                        .map(|a| a.into_owned())
                        .unwrap_or(async_graphql::Value::Null)),
                    Context::PushArgs { expr, and_then } => {
                        let args = expr.eval(ctx).await?;
                        let ctx = &mut ctx.with_args(args);
                        and_then.eval(ctx).await
                    }
                    Context::PushValue { expr, and_then } => {
                        let value = expr.eval(ctx).await?;
                        let ctx = &mut ctx.with_value(value);
                        and_then.eval(ctx).await
                    }
                },
                IR::Path(input, path) => {
                    let inp = &input.eval(ctx).await?;
                    Ok(inp
                        .get_path(path)
                        .unwrap_or(&async_graphql::Value::Null)
                        .clone())
                }
                IR::Dynamic(value) => Ok(value.render_value(ctx)),
                IR::Protect(expr) => {
                    ctx.request_ctx
                        .auth_ctx
                        .validate(ctx.request_ctx)
                        .await
                        .to_result()?;
                    expr.eval(ctx).await
                }
                IR::IO(operation) => operation.execute(ctx).await,
                IR::Cache(Cache { max_age, io }) => {
                    let io = io.deref();
                    let key = io.cache_key(ctx);
                    if let Some(key) = key {
                        if let Some(val) = ctx.request_ctx.runtime.cache.get(&key).await? {
                            Ok(val)
                        } else {
                            let val = io.execute(ctx).await?;
                            ctx.request_ctx
                                .runtime
                                .cache
                                .set(key, val.clone(), max_age.to_owned())
                                .await?;
                            Ok(val)
                        }
                    } else {
                        io.execute(ctx).await
                    }
                }
                IR::Map(Map { input, map }) => {
                    let value = input.eval(ctx).await?;
                    if let ConstValue::String(key) = value {
                        if let Some(value) = map.get(&key) {
                            Ok(ConstValue::String(value.to_owned()))
                        } else {
                            Err(EvaluationError::ExprEvalError(format!(
                                "Can't find mapped key: {}.",
                                key
                            )))
                        }
                    } else {
                        Err(EvaluationError::ExprEvalError(
                            "Mapped key must be string value.".to_owned(),
                        ))
                    }
                }
            }
        })
    }
}
