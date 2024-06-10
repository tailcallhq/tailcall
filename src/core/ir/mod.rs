mod cache;
mod discriminator;
mod error;
mod eval;
mod evaluation_context;
mod graphql_operation_context;
mod io;
mod modify;
mod resolver_context_like;

use core::future::Future;
use std::fmt::Debug;
use std::pin::Pin;

use async_graphql_value::ConstValue;
pub use cache::*;
pub use discriminator::{Discriminator, TypeName};
pub use error::*;
pub use eval::*;
pub use evaluation_context::EvaluationContext;
pub use graphql_operation_context::GraphQLOperationContext;
pub use io::*;
pub use resolver_context_like::{EmptyResolverContext, ResolverContext, ResolverContextLike};
use strum_macros::Display;

use crate::core::blueprint::DynamicValue;
use crate::core::json::JsonLike;
use crate::core::serde_value_ext::ValueExt;

#[derive(Clone, Debug, Display)]
pub enum IR {
    Context(Context),
    Dynamic(DynamicValue),
    #[strum(to_string = "{0}")]
    IO(IO),
    Cache(Cache),
    Path(Box<IR>, Vec<String>),
    Protect(Box<IR>),
    Discriminate(Discriminator, Box<IR>),
}

#[derive(Clone, Debug)]
pub enum Context {
    Value,
    Path(Vec<String>),
    PushArgs { expr: Box<IR>, and_then: Box<IR> },
    PushValue { expr: Box<IR>, and_then: Box<IR> },
}

impl IR {
    pub fn and_then(self, next: Self) -> Self {
        IR::Context(Context::PushArgs { expr: Box::new(self), and_then: Box::new(next) })
    }

    pub fn with_args(self, args: IR) -> Self {
        IR::Context(Context::PushArgs { expr: Box::new(args), and_then: Box::new(self) })
    }
}

impl Eval for IR {
    #[tracing::instrument(skip_all, fields(otel.name = %self), err)]
    fn eval<'slf, 'ctx, Ctx: ResolverContextLike + Sync + Send>(
        &'slf self,
        ctx: &'ctx mut EvaluationContext<'slf, Ctx>,
    ) -> Pin<Box<dyn Future<Output = Result<ConstValue, EvaluationError>> + 'ctx + Send>> {
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
                        let ctx = &mut ctx.with_args(args).clone();
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
                IR::IO(operation) => operation.eval(ctx).await,
                IR::Cache(cached) => cached.eval(ctx).await,
                IR::Discriminate(discriminator, expr) => expr.eval(ctx).await.and_then(|value| {
                    let type_name = discriminator.resolve_type(&value)?;

                    ctx.set_type_name(type_name);

                    Ok(value)
                }),
            }
        })
    }
}
