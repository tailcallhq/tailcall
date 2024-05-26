use core::future::Future;
use std::fmt::{Debug, Display};
use std::pin::Pin;
use std::sync::Arc;

use async_graphql::ErrorExtensions;
use async_graphql_value::ConstValue;
use thiserror::Error;

use super::{Eval, EvaluationContext, ResolverContextLike, IO};
use crate::core::auth;
use crate::core::blueprint::DynamicValue;
use crate::core::ir::cache::Cache;
use crate::core::json::JsonLike;
use crate::core::serde_value_ext::ValueExt;

#[derive(Clone, Debug)]
pub enum IR {
    Context(Context),
    Dynamic(DynamicValue),
    IO(IO),
    Cache(Cache),
    Path(Box<IR>, Vec<String>),
    Protect(Box<IR>),
}

impl Display for IR {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IR::Context(_) => write!(f, "Context"),
            IR::Dynamic(_) => write!(f, "Literal"),
            IR::IO(io) => write!(f, "{io}"),
            IR::Cache(_) => write!(f, "Cache"),
            IR::Path(_, _) => write!(f, "Input"),
            IR::Protect(expr) => write!(f, "Protected({expr})"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Context {
    Value,
    Path(Vec<String>),
    PushArgs { expr: Box<IR>, and_then: Box<IR> },
    PushValue { expr: Box<IR>, and_then: Box<IR> },
}

#[derive(Debug, Error, Clone)]
pub enum EvaluationError {
    #[error("IOException: {0}")]
    IOException(String),

    #[error("gRPC Error: status: {grpc_code}, description: `{grpc_description}`, message: `{grpc_status_message}`")]
    GRPCError {
        grpc_code: i32,
        grpc_description: String,
        grpc_status_message: String,
        grpc_status_details: ConstValue,
    },

    #[error("APIValidationError: {0:?}")]
    APIValidationError(Vec<String>),

    #[error("ExprEvalError: {0}")]
    ExprEvalError(String),

    #[error("DeserializeError: {0}")]
    DeserializeError(String),

    #[error("Authentication Failure: {0}")]
    AuthError(auth::error::Error),
}

// TODO: remove conversion from anyhow and don't use anyhow to pass errors
// since it loses potentially valuable information that could be later provided
// in the error extensions
impl From<anyhow::Error> for EvaluationError {
    fn from(value: anyhow::Error) -> Self {
        match value.downcast::<EvaluationError>() {
            Ok(err) => err,
            Err(err) => EvaluationError::IOException(err.to_string()),
        }
    }
}

impl From<Arc<anyhow::Error>> for EvaluationError {
    fn from(error: Arc<anyhow::Error>) -> Self {
        match error.downcast_ref::<EvaluationError>() {
            Some(err) => err.clone(),
            None => EvaluationError::IOException(error.to_string()),
        }
    }
}

impl ErrorExtensions for EvaluationError {
    fn extend(&self) -> async_graphql::Error {
        async_graphql::Error::new(format!("{}", self)).extend_with(|_err, e| {
            if let EvaluationError::GRPCError {
                grpc_code,
                grpc_description,
                grpc_status_message,
                grpc_status_details,
            } = self
            {
                e.set("grpcCode", *grpc_code);
                e.set("grpcDescription", grpc_description);
                e.set("grpcStatusMessage", grpc_status_message);
                e.set("grpcStatusDetails", grpc_status_details.clone());
            }
        })
    }
}

impl<'a> From<crate::core::valid::ValidationError<&'a str>> for EvaluationError {
    fn from(value: crate::core::valid::ValidationError<&'a str>) -> Self {
        EvaluationError::APIValidationError(
            value
                .as_vec()
                .iter()
                .map(|e| e.message.to_owned())
                .collect(),
        )
    }
}

impl From<auth::error::Error> for EvaluationError {
    fn from(value: auth::error::Error) -> Self {
        EvaluationError::AuthError(value)
    }
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
    fn eval<'a, Ctx: ResolverContextLike<'a> + Sync + Send>(
        &'a self,
        ctx: EvaluationContext<'a, Ctx>,
    ) -> Pin<Box<dyn Future<Output = Result<ConstValue, EvaluationError>> + 'a + Send>> {
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
                        let args = expr.eval(ctx.clone()).await?;
                        let ctx = ctx.with_args(args).clone();
                        and_then.eval(ctx).await
                    }
                    Context::PushValue { expr, and_then } => {
                        let value = expr.eval(ctx.clone()).await?;
                        let ctx = ctx.with_value(value);
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
                IR::Dynamic(value) => Ok(value.render_value(&ctx)),
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
            }
        })
    }
}
