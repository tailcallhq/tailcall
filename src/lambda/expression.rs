use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use anyhow::Result;
use async_graphql_value::Name;
use http_cache_semantics::RequestLike;
use indexmap::IndexMap;
use serde::Serialize;
use serde_json::Value;
use thiserror::Error;

use crate::endpoint::Endpoint;
#[cfg(feature = "unsafe-js")]
use crate::javascript;
use crate::json::JsonLike;
use crate::lambda::EvaluationContext;

#[derive(Clone, Debug)]
pub enum Expression {
  Context(Context),
  Literal(Value), // TODO: this should async_graphql::Value
  EqualTo(Box<Expression>, Box<Expression>),
  Unsafe(Box<Expression>, Operation),
  Input(Box<Expression>, Vec<String>),
}

#[derive(Clone, Debug)]
pub enum Context {
  Value,
  Path(Vec<String>),
}

#[derive(Clone, Debug)]
pub enum Operation {
  Endpoint(Endpoint),
  JS(String),
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

fn to_body(value: HashMap<String, Vec<&async_graphql::Value>>) -> async_graphql::Value {
  let mut map = IndexMap::new();
  for (k, v) in value {
    let list = Vec::from_iter(v.iter().map(|v| v.to_owned().to_owned()));
    map.insert(Name::new(k), async_graphql::Value::List(list));
  }
  async_graphql::Value::Object(map)
}

impl Expression {
  pub fn eval<'a>(
    &'a self,
    ctx: &'a EvaluationContext<'a>,
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
        Expression::Unsafe(input, operation) => {
          let input = input.eval(ctx).await?;
          match operation {
            Operation::Endpoint(endpoint) => {
              let req = endpoint.to_request(&input, ctx)?;
              let url = req.uri().clone();
              let is_get = req.method() == reqwest::Method::GET;
              // Attempt to short circuit GET request
              if is_get {
                if let Some(cached) = ctx.req_ctx.get(&url) {
                  if let Some(key) = endpoint.batch_key() {
                    return Ok(cached.body.get_key(key).cloned().unwrap_or(async_graphql::Value::Null));
                  }
                  return Ok(cached.body);
                }
              }

              // Prepare for HTTP calls
              let mut res = ctx
                .req_ctx
                .execute(req)
                .await
                .map_err(|e| EvaluationError::IOException(e.to_string()))?;

              // Handle N + 1 batching
              if let Some(batch) = endpoint.batch.as_ref() {
                let path = batch.path();
                res.body = to_body(res.body.group_by(path));
              } else {
                // Enable HTTP validation if batching is disabled
                if ctx.req_ctx.server.enable_http_validation() {
                  endpoint.output.validate(&res.body).map_err(EvaluationError::from)?;
                }
              }

              // Insert into cache for future requests
              if is_get {
                ctx.req_ctx.insert(url, res.clone());
              }

              // If batching is enabled pick the batch key
              if let Some(batch) = endpoint.batch.as_ref() {
                Ok(
                  res
                    .body
                    .get_key(batch.key())
                    .cloned()
                    .unwrap_or(async_graphql::Value::Null),
                )
              } else {
                Ok(res.body)
              }
            }
            Operation::JS(script) => {
              let result;
              #[cfg(not(feature = "unsafe-js"))]
              {
                let _ = script;
                result = Err(EvaluationError::JSException("JS execution is disabled".to_string()).into());
              }

              #[cfg(feature = "unsafe-js")]
              {
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
