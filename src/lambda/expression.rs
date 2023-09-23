use std::future::Future;
use std::pin::Pin;

use anyhow::Result;
use async_graphql::InputType;
use serde::Serialize;
use serde_json::Value;
use thiserror::Error;

use crate::endpoint::Endpoint;
use crate::http::{EndpointKey, Method};
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
              // TODO: header forwarding should happen inside of endpoint
              let env = &ctx.req_ctx.server.vars.to_value();
              let url = endpoint.get_url(&input, Some(env), ctx.args().as_ref(), &ctx.req_ctx.req_headers)?;

              if endpoint.method == Method::GET {
                let match_key_value = endpoint
                  .batch_key()
                  .map(|key| {
                    input
                      .get_path(&[key.to_string()])
                      .unwrap_or(&async_graphql::Value::Null)
                  })
                  .unwrap_or(&async_graphql::Value::Null);
                let key = EndpointKey {
                  url: url.clone(),
                  headers: ctx.req_ctx.req_headers.clone(),
                  method: endpoint.method.clone(),
                  match_key_value: match_key_value.clone(),
                  match_path: endpoint.batch_path().to_vec(),
                  batching_enabled: endpoint.is_batched(),
                  list: endpoint.list.unwrap_or(false),
                };
                let value = ctx
                  .req_ctx
                  .data_loader
                  .load_one(key)
                  .await
                  .map_err(|e| EvaluationError::IOException(e.to_string()))?
                  .unwrap_or_default();
                if ctx.req_ctx.server.enable_http_validation() {
                  endpoint.output.validate(&value.body).map_err(EvaluationError::from)?;
                }
                Ok(value.body)
              } else {
                let req = endpoint.to_request(&input, Some(env), ctx.args().as_ref(), ctx.headers())?;
                let client = crate::http::HttpClient::default();
                let value = client
                  .execute(req)
                  .await
                  .map_err(|e| EvaluationError::IOException(e.to_string()))?;
                if ctx.req_ctx.server.enable_http_validation() {
                  endpoint.output.validate(&value.body).map_err(EvaluationError::from)?;
                }
                Ok(value.body)
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
