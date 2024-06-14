use std::future::Future;
use std::pin::Pin;

use serde_json_borrow::{OwnedValue, Value};

use crate::core::ir::jit::execute::IOExit;
use crate::core::ir::jit::store::Store;
use crate::core::ir::{
    CacheKey, Context, Eval, EvalSync, EvaluationContext, EvaluationError, IoId,
    ResolverContextLike, IR,
};
use crate::core::serde_value_ext::ValueExt;

fn into_owned(value: async_graphql::Value) -> Result<OwnedValue, EvaluationError> {
    let value = value
        .into_json()
        .map_err(|e| EvaluationError::DeserializeError(e.to_string()))?;

    let owned_val = OwnedValue::from_string(value.to_string()).map_err(|e| {
        EvaluationError::DeserializeError(format!("Failed to deserialize IO value: {}", e))
    })?;
    Ok(owned_val)
}

fn into_const(value: &Value<'_>) -> Result<async_graphql::Value, EvaluationError> {
    let value = serde_json::Value::from(value);
    let cv = async_graphql::Value::from_json(value).map_err(|e| {
        EvaluationError::DeserializeError(format!("Failed to deserialize IO value: {}", e))
    })?;
    Ok(cv)
}

fn get_path<'a, T: AsRef<str>>(mut val: &'a Value<'a>, path: &[T]) -> Option<&'a Value<'a>> {
    for token in path {
        val = match val {
            Value::Array(seq) => {
                let index = token.as_ref().parse::<usize>().ok()?;
                seq.get(index)?
            }
            Value::Object(map) => map.get(token.as_ref())?,
            _ => return None,
        };
    }
    Some(val)
}

impl Eval<IOExit> for IR {
    fn eval<'a, Ctx: ResolverContextLike<'a> + Sync + Send>(
        &'a self,
        ctx: EvaluationContext<'a, Ctx>,
    ) -> Pin<Box<dyn Future<Output = Result<IOExit, EvaluationError>> + 'a + Send>>
    where
        IOExit: 'a,
    {
        Box::pin(async move {
            match self {
                IR::Context(op) => match op {
                    Context::Value => Ok(IOExit::new(
                        into_owned(ctx.value().cloned().unwrap_or(async_graphql::Value::Null))?,
                        None,
                    )),
                    Context::Path(path) => {
                        let val = ctx
                            .path_value(path)
                            .map(|a| a.into_owned())
                            .unwrap_or(async_graphql::Value::Null);

                        Ok(IOExit::new(into_owned(val)?, None))
                    }
                    Context::PushArgs { expr, and_then } => {
                        let args = into_const(
                            <IR as Eval<IOExit>>::eval::<'_, Ctx>(expr, ctx.clone())
                                .await?
                                .data
                                .get_value(),
                        )?;
                        let ctx = ctx.with_args(args).clone();
                        and_then.eval(ctx).await
                    }
                    Context::PushValue { expr, and_then } => {
                        let value = into_const(
                            <IR as Eval<IOExit>>::eval::<'_, Ctx>(expr, ctx.clone())
                                .await?
                                .data
                                .get_value(),
                        )?;
                        let ctx = ctx.with_value(value);
                        and_then.eval(ctx).await
                    }
                },
                IR::Path(input, path) => {
                    let val = <IR as Eval<IOExit>>::eval::<'_, Ctx>(input, ctx).await?;
                    let io_id = val.id;
                    let val = get_path(val.data.get_value(), path)
                        .unwrap_or(&Value::Null)
                        .clone();

                    let val = OwnedValue::from_string(val.to_string()).map_err(|e| {
                        EvaluationError::DeserializeError(format!(
                            "Failed to deserialize IO value: {}",
                            e
                        ))
                    })?;

                    Ok(IOExit::new(val, io_id))
                }
                IR::Dynamic(value) => {
                    let val = value.render_value(&ctx);
                    let val = into_owned(val)?;
                    Ok(IOExit::new(val, None))
                }
                IR::Protect(expr) => {
                    ctx.request_ctx
                        .auth_ctx
                        .validate(ctx.request_ctx)
                        .await
                        .to_result()?;
                    expr.eval(ctx).await
                }
                IR::IO(operation) => {
                    let io_id = operation
                        .cache_key(&ctx)
                        .ok_or(EvaluationError::ExprEvalError(
                            "Unable to generate cache key".to_string(),
                        ))?;
                    let value = operation.eval(ctx).await.map_err(|e| {
                        EvaluationError::ExprEvalError(format!("Unable to evaluate: {}", e))
                    })?;
                    let owned_val = into_owned(value)?;

                    Ok(IOExit::new(owned_val, Some(io_id)))
                }
                IR::Cache(cached) => {
                    let val = cached.eval(ctx).await?;
                    let val = into_owned(val)?;
                    Ok(IOExit::new(val, None))
                }
            }
        })
    }
}

impl EvalSync<OwnedValue> for IR {
    fn eval_sync<'a, Ctx: ResolverContextLike<'a> + Sync + Send>(
        &'a self,
        store: &'a Store<IoId, OwnedValue>,
        ctx: EvaluationContext<'a, Ctx>,
    ) -> Result<OwnedValue, EvaluationError> {
        match self {
            IR::Context(op) => match op {
                Context::Value => Ok(into_owned(
                    ctx.value().cloned().unwrap_or(async_graphql::Value::Null),
                )?),
                Context::Path(path) => {
                    let val = ctx
                        .path_value(path)
                        .map(|a| a.into_owned())
                        .unwrap_or(async_graphql::Value::Null);

                    into_owned(val)
                }
                Context::PushArgs { expr, and_then } => {
                    let args = into_const(expr.eval_sync(store, ctx.clone())?.get_value())?;
                    let ctx = ctx.with_args(args).clone();
                    and_then.eval_sync(store, ctx)
                }
                Context::PushValue { expr, and_then } => {
                    let value = into_const(expr.eval_sync(store, ctx.clone())?.get_value())?;
                    let ctx = ctx.with_value(value);
                    and_then.eval_sync(store, ctx)
                }
            },
            IR::Dynamic(value) => {
                let val = value.render_value(&ctx);
                into_owned(val)
            }
            IR::IO(io) => {
                let key = io.cache_key(&ctx).ok_or(EvaluationError::ExprEvalError(
                    "Unable to generate cache key".to_string(),
                ))?;
                if let Some(val) = store.get(&key) {
                    Ok(val.clone())
                } else {
                    Ok(OwnedValue::from_string(serde_json::Value::Null.to_string()).unwrap())
                }
            }
            IR::Path(input, path) => {
                let val = input.eval_sync(store, ctx)?;
                let val = get_path(val.get_value(), path)
                    .unwrap_or(&Value::Null)
                    .clone();

                let val = OwnedValue::from_string(val.to_string()).map_err(|e| {
                    EvaluationError::DeserializeError(format!(
                        "Failed to deserialize IO value: {}",
                        e
                    ))
                })?;

                Ok(val)
            }
            _ => {
                todo!("Cache and Protect needs async")
            }
        }
    }
}
