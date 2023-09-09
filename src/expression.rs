use std::future::Future;
use std::pin::Pin;

use anyhow::Result;
use async_graphql::InputType;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::endpoint::Endpoint;
use crate::evaluation_context::EvaluationContext;
use crate::http::EndpointKey;
use crate::http::Method;
#[cfg(feature = "unsafe-js")]
use crate::javascript;
use crate::json::JsonLike;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Expression {
    Context(Context),
    Literal(Value),
    // TODO: this should async_graphql::Value
    EqualTo(Box<Expression>, Box<Expression>),
    Unsafe(Box<Expression>, Operation),
    Input(Box<Expression>, Vec<String>),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Context {
    Value,
    Path(Vec<String>),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
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
                    let headers: Vec<(String, String)> = ctx
                        .get_headers()
                        .clone()
                        .into_iter()
                        .map(|(k, v)| (k.to_string(), v.to_string()))
                        .collect();
                    match operation {
                        Operation::Endpoint(endpoint) => {
                            let url = endpoint.get_url(
                                &input,
                                Some(&ctx.env.to_value()),
                                ctx.args().as_ref(),
                                &headers.to_owned(),
                            )?;

                            if endpoint.method == Method::GET && ctx.server.enable_join_cache() {
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
                                    headers,
                                    method: endpoint.method.clone(),
                                    match_key_value: match_key_value.clone(),
                                    match_path: endpoint.batch_path().to_vec(),
                                    batching_enabled: endpoint.is_batched(),
                                    list: endpoint.list.unwrap_or(false),
                                };
                                let value = ctx
                                    .data_loader
                                    .load_one(key)
                                    .await
                                    .map_err(|e| EvaluationError::IOException(e.to_string()))?
                                    .unwrap_or_default();
                                if ctx.server.enable_http_validation() {
                                    endpoint.output.validate(&value.body).map_err(EvaluationError::from)?;
                                }
                                Ok(value.body)
                            } else {
                                let req = endpoint.into_request(
                                    &input,
                                    Some(&ctx.env.to_value()),
                                    ctx.args().as_ref(),
                                    &headers.to_owned(),
                                );
                                let value = ctx
                                    .client
                                    .execute(req)
                                    .await
                                    .map_err(|e| EvaluationError::IOException(e.to_string()))?;
                                if ctx.server.enable_http_validation() {
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
                                result =
                                    Err(EvaluationError::JSException("JS execution is disabled".to_string()).into());
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
