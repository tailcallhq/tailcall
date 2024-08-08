use std::borrow::Cow;
use std::future::Future;
use std::ops::Deref;

use async_graphql::Name;
use async_graphql_value::ConstValue;
use indexmap::IndexMap;

use super::eval_io::eval_io;
use super::model::{Cache, CacheKey, Map, IR};
use super::{Error, EvalContext, ResolverContextLike};
use crate::core::ir::model::{InputTransforms, TransformKey};
use crate::core::json::JsonLike;
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
                IR::ModifyInput(input_transforms) => {
                    if let Some(args) = ctx.path_arg::<&str>(&[]) {
                        let args: IndexMap<Name, ConstValue> = args
                            .as_object()
                            .unwrap()
                            .iter()
                            .map(|(name, value)| {
                                if name.to_string().eq(&input_transforms.arg_name) {
                                    (
                                        name.clone(),
                                        handle_args(
                                            value,
                                            input_transforms,
                                            &input_transforms.arg_type,
                                        )
                                        .into_owned(),
                                    )
                                } else {
                                    (name.clone(), value.clone())
                                }
                            })
                            .collect();

                        fn handle_args<'a>(
                            args: &'a ConstValue,
                            input_transforms: &'a InputTransforms,
                            type_of: &'a str,
                        ) -> Cow<'a, ConstValue> {
                            match &args {
                                ConstValue::Null => Cow::Borrowed(args),
                                ConstValue::Number(_) => Cow::Borrowed(args),
                                ConstValue::String(_) => Cow::Borrowed(args),
                                ConstValue::Boolean(_) => Cow::Borrowed(args),
                                ConstValue::Binary(_) => Cow::Borrowed(args),
                                ConstValue::Enum(_) => Cow::Borrowed(args),
                                ConstValue::List(items) => {
                                    let value = ConstValue::List(
                                        items
                                            .iter()
                                            .cloned()
                                            .map(move |item| {
                                                handle_args(&item, input_transforms, type_of)
                                                    .into_owned()
                                            })
                                            .collect::<Vec<_>>(),
                                    );
                                    Cow::Owned(value)
                                }
                                ConstValue::Object(obj) => {
                                    let mut new_map = IndexMap::new();

                                    for (name, item) in obj {
                                        let key = TransformKey::from_str(
                                            type_of.to_string(),
                                            name.to_string(),
                                        );
                                        let type_new = input_transforms.subfield_types.get(&key);
                                        let name_new = input_transforms.subfield_renames.get(&key);

                                        match (type_new, name_new) {
                                            (None, None) => new_map.insert(
                                                name.clone(),
                                                handle_args(item, input_transforms, type_of),
                                            ),
                                            (None, Some(name_new)) => new_map.insert(
                                                Name::new(name_new),
                                                handle_args(item, input_transforms, type_of),
                                            ),
                                            (Some(type_new), None) => new_map.insert(
                                                name.clone(),
                                                handle_args(item, input_transforms, type_new),
                                            ),
                                            (Some(type_new), Some(name_new)) => new_map.insert(
                                                Name::new(name_new),
                                                handle_args(item, input_transforms, type_new),
                                            ),
                                        };
                                    }

                                    let new_map = new_map
                                        .into_iter()
                                        .map(|(name, value)| (name, value.into_owned()))
                                        .collect();

                                    Cow::Owned(ConstValue::Object(new_map))
                                }
                            }
                        }

                        Ok(ConstValue::Object(args))
                    } else {
                        Ok(ConstValue::Null)
                    }
                }
            }
        })
    }
}
