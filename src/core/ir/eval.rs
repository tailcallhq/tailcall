use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;

use async_graphql_value::ConstValue;

use super::eval_io::eval_io;
use super::model::{Cache, CacheKey, Map, IR};
use super::{Error, EvalContext, ResolverContextLike};
use crate::core::json::JsonLike;
use crate::core::serde_value_ext::ValueExt;

impl IR {
    #[tracing::instrument(skip_all, fields(otel.name = %self), err)]
    pub fn eval<'a, 'b, Ctx>(
        &'a self,
        ctx: &'b mut EvalContext<'a, Ctx>,
    ) -> Pin<Box<dyn Future<Output = Result<ConstValue, Error>> + Send + 'b>>
    where
        Ctx: ResolverContextLike + Sync,
    {
        Box::pin(async move {
            match self {
                IR::ContextPath(path) => Ok(ctx
                    .path_value(path)
                    .map(|a| a.into_owned())
                    .unwrap_or(async_graphql::Value::Null)),
                IR::Path(input, path) => {
                    let inp = input.eval(ctx).await?;
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
                IR::IO(io) => eval_io(io, ctx).await,
                IR::Cache(Cache { max_age, io }) => {
                    let io = io.deref();
                    let key = io.cache_key(ctx);
                    if let Some(key) = key {
                        if let Some(val) = ctx.request_ctx.runtime.cache.get(&key).await? {
                            Ok(val)
                        } else {
                            let val = eval_io(io, ctx).await?;
                            ctx.request_ctx
                                .runtime
                                .cache
                                .set(key, val.clone(), max_age.to_owned())
                                .await?;
                            Ok(val)
                        }
                    } else {
                        eval_io(io, ctx).await
                    }
                }
                IR::Map(Map { input, map }) => {
                    let value = input.eval(ctx).await?;
                    if let ConstValue::String(key) = value {
                        if let Some(value) = map.get(&key) {
                            Ok(ConstValue::String(value.to_owned()))
                        } else {
                            Err(Error::ExprEvalError(format!(
                                "Can't find mapped key: {}.",
                                key
                            )))
                        }
                    } else {
                        Err(Error::ExprEvalError(
                            "Mapped key must be string value.".to_owned(),
                        ))
                    }
                }
                IR::Pipe(first, second) => {
                    let args = first.eval(&mut ctx.clone()).await?;
                    let ctx = &mut ctx.with_args(args);
                    second.eval(ctx).await
                }
                IR::Discriminate(discriminator, expr) => expr.eval(ctx).await.and_then(|value| {
                    let type_name = discriminator.resolve_type(&value)?;

                    ctx.set_type_name(type_name);

                    Ok(value)
                }),
            }
        })
    }
}
