use std::future::Future;
use std::ops::Deref;

use async_graphql_value::ConstValue;
use futures_util::future::join_all;
use indexmap::IndexMap;

use super::eval_io::eval_io;
use super::model::{Cache, CacheKey, Map, IR};
use super::{Error, EvalContext, ResolverContextLike, TypedValue};
use crate::core::json::{JsonLike, JsonLikeList, JsonObjectLike};
use crate::core::serde_value_ext::ValueExt;

// Fake trait to capture proper lifetimes.
// see discussion https://users.rust-lang.org/t/rpitit-allows-more-flexible-code-in-comparison-with-raw-rpit-in-inherit-impl/113417
// TODO: could be removed after migrating to 2024 edition
pub trait Captures<T: ?Sized> {}
impl<T: ?Sized, U: ?Sized> Captures<T> for U {}

impl IR {
    #[tracing::instrument(skip_all, fields(otel.name = %self), err)]
    pub fn eval<'a, 'b, Ctx>(
        &'a self,
        ctx: &'b mut EvalContext<'a, Ctx>,
    ) -> impl Future<Output = Result<ConstValue, Error>> + Send + Captures<&'b &'a ()>
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
                            Err(Error::ExprEval(format!("Can't find mapped key: {}.", key)))
                        }
                    } else {
                        Err(Error::ExprEval(
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
                    let value = value.map(&mut |mut value| {
                        if value.get_type_name().is_some() {
                            // if typename is already present in value just reuse it instead
                            // of recalculating from scratch
                            return Ok(value);
                        }

                        let type_name = discriminator.resolve_type(&value)?;

                        value.set_type_name(type_name.to_string())?;

                        anyhow::Ok(value)
                    })?;

                    Ok(value)
                }),
                IR::EntityResolver(map) => {
                    let representations = ctx.path_arg(&["representations"]);

                    let representations = representations
                        .as_ref()
                        .and_then(|repr| repr.as_array())
                        .ok_or(Error::EntityResolver(
                            "expected `representations` arg as an array of _Any".to_string(),
                        ))?;

                    let mut tasks = Vec::with_capacity(representations.len());

                    for repr in representations {
                        // TODO: combine errors, instead of fail fast?
                        let type_name = repr.get_type_name().ok_or(Error::EntityResolver(
                            "expected __typename to be the part of the representation".to_string(),
                        ))?;

                        let ir = map.get(type_name).ok_or(Error::EntityResolver(format!(
                            "Cannot find a resolver for type: `{type_name}`"
                        )))?;

                        // pass the input for current representation as value in context
                        // TODO: can we drop clone?
                        let mut ctx = ctx.with_value(repr.clone());

                        tasks.push(async move {
                            ir.eval(&mut ctx).await.and_then(|mut value| {
                                // set typename explicitly to reuse it if needed
                                value.set_type_name(type_name.to_owned())?;
                                Ok(value)
                            })
                        });
                    }

                    let result = join_all(tasks).await;

                    let entities = result.into_iter().collect::<Result<_, _>>()?;

                    Ok(ConstValue::List(entities))
                }
                IR::Service(sdl) => {
                    let mut obj = IndexMap::new();

                    obj.insert_key("sdl", ConstValue::string(sdl.into()));

                    Ok(ConstValue::object(obj))
                }
            }
        })
    }
}
