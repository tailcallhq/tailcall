use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use anyhow::Result;
use async_graphql::dataloader::{DataLoader, NoCache};
use serde::Serialize;
use serde_json::Value;
use thiserror::Error;

use super::ResolverContextLike;
use crate::config::group_by::GroupBy;
use crate::http::{max_age, DefaultHttpClient, HttpDataLoader};
#[cfg(feature = "unsafe-js")]
use crate::javascript;
use crate::json::JsonLike;
use crate::lambda::EvaluationContext;
use crate::request_template::RequestTemplate;

#[derive(Clone, Debug)]
pub enum Expression {
  Context(Context),
  Literal(Value), // TODO: this should async_graphql::Value
  EqualTo(Box<Expression>, Box<Expression>),
  Unsafe(Operation),
  Input(Box<Expression>, Vec<String>),
}

#[derive(Clone, Debug)]
pub enum Context {
  Value,
  Path(Vec<String>),
}

#[derive(Clone)]
pub enum Operation {
  Endpoint(
    RequestTemplate,
    Option<GroupBy>,
    Option<Arc<DataLoader<HttpDataLoader<DefaultHttpClient>, NoCache>>>,
  ),
  JS(Box<Expression>, String),
}

impl Debug for Operation {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Operation::Endpoint(req_template, group_by, dl) => f
        .debug_struct("Endpoint")
        .field("req_template", req_template)
        .field("group_by", group_by)
        .field("dl", &dl.clone().map(|a| a.clone().loader().batched.clone()))
        .finish(),
      Operation::JS(input, script) => f
        .debug_struct("JS")
        .field("input", input)
        .field("script", script)
        .finish(),
    }
  }
}

#[derive(Debug, Error, Serialize)]
pub enum EvaluationError {
  #[error("IOException: {0}")]
  IOException(String),

  #[error("JSException: {0}")]
  JSException(String),

  #[error("APIValidationError: {0:?}")]
  APIValidationError(Vec<String>),
}

impl<'a> From<crate::valid::ValidationError<&'a str>> for EvaluationError {
  fn from(_value: crate::valid::ValidationError<&'a str>) -> Self {
    EvaluationError::APIValidationError(_value.as_vec().iter().map(|e| e.message.to_owned()).collect())
  }
}

impl Expression {
  pub fn eval<'a, Ctx: ResolverContextLike<'a> + Sync + Send>(
    &'a self,
    ctx: &'a EvaluationContext<'a, Ctx>,
  ) -> Pin<Box<dyn Future<Output = Result<async_graphql::Value>> + 'a + Send>> {
    Box::pin(async move {
      match self {
        Expression::Context(op) => match op {
          Context::Value => Ok(ctx.value().cloned().unwrap_or(async_graphql::Value::Null)),
          Context::Path(path) => Ok(ctx.path_value(path).cloned().unwrap_or(async_graphql::Value::Null)),
        },
        Expression::Input(input, path) => {
          let inp = &input.eval(ctx).await?;
          Ok(inp.get_path(path).unwrap_or(&async_graphql::Value::Null).clone())
        }
        Expression::Literal(value) => Ok(serde_json::from_value(value.clone())?),
        Expression::EqualTo(left, right) => Ok(async_graphql::Value::from(
          left.eval(ctx).await? == right.eval(ctx).await?,
        )),
        Expression::Unsafe(operation) => {
          match operation {
            Operation::Endpoint(req_template, _, dl) => {
              let req = req_template.to_request(ctx)?;
              let is_get = req.method() == reqwest::Method::GET;
              // Attempt to short circuit GET request
              if is_get && ctx.req_ctx.upstream.batch.is_some() {
                let headers = ctx
                  .req_ctx
                  .upstream
                  .batch
                  .clone()
                  .map(|s| s.headers)
                  .unwrap_or_default();
                let endpoint_key = crate::http::DataLoaderRequest::new(req, headers);
                let resp = dl
                  .as_ref()
                  .unwrap()
                  .load_one(endpoint_key)
                  .await
                  .map_err(|e| EvaluationError::IOException(e.to_string()))?
                  .unwrap_or_default();
                if ctx.req_ctx.server.get_enable_cache_control() && resp.status.is_success() {
                  if let Some(max_age) = max_age(&resp) {
                    ctx.req_ctx.set_min_max_age(max_age.as_secs());
                  }
                }
                return Ok(resp.body);
              }

              // Prepare for HTTP calls
              let res = ctx
                .req_ctx
                .execute(req)
                .await
                .map_err(|e| EvaluationError::IOException(e.to_string()))?;
              if ctx.req_ctx.server.get_enable_http_validation() {
                req_template
                  .endpoint
                  .output
                  .validate(&res.body)
                  .map_err(EvaluationError::from)?;
              }
              if ctx.req_ctx.server.get_enable_cache_control() && res.status.is_success() {
                if let Some(max_age) = max_age(&res) {
                  ctx.req_ctx.set_min_max_age(max_age.as_secs());
                }
              }
              Ok(res.body)
            }
            Operation::JS(input, script) => {
              let result;
              #[cfg(not(feature = "unsafe-js"))]
              {
                let _ = script;
                let _ = input;
                result = Err(EvaluationError::JSException("JS execution is disabled".to_string()).into());
              }

              #[cfg(feature = "unsafe-js")]
              {
                let input = input.eval(ctx).await?;
                result = javascript::execute_js(script, input, Some(ctx.timeout))
                  .map_err(|e| EvaluationError::JSException(e.to_string()).into());
              }
              result
            }
          }
        }
      }
    })
  }
}
