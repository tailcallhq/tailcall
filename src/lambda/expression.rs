use std::future::Future;
use std::pin::Pin;

use anyhow::Result;
use http_cache_semantics::RequestLike;
use serde::Serialize;
use serde_json::Value;
use thiserror::Error;

#[cfg(feature = "unsafe-js")]
use crate::javascript;
use crate::json::JsonLike;
use crate::lambda::EvaluationContext;
use crate::request_template::RequestTemplate;

#[derive(Debug, Error, Serialize)]
pub enum EvaluationError {
    #[error("IOException: {0}")]
    IOException(String),

    #[error("JSException: {0}")]
    JSException(String),

    #[error("APIValidationError: {0:?}")]
    APIValidationError(Vec<String>),
}

#[derive(Clone, Debug)]
pub enum Operation {
    Endpoint(RequestTemplate),
    JS(Box<Expression>, String),
}

impl<'a> Expression {
    pub fn eval<'a>(
        &'a self,
        ctx: &'a EvaluationContext<'a>,
    ) -> Pin<Box<dyn Future<Output = Result<async_graphql::Value>> + 'a + Send>> {
        Box::pin(async move {
            match self {
                Expression::Unsafe(operation) => {
                    match operation {
                        Operation::Endpoint(req_template) => {
                            let req = req_template.to_request(ctx)?;
                            let url = req.uri().clone();
                            let is_get = req.method() == reqwest::Method::GET;
                            // Attempt to short circuit GET request
                            if is_get {
                                if let Some(cached) = ctx.req_ctx.cache.get(&url) {
                                    return Ok(cached.body);
                                }
                            }

                            // Prepare for HTTP calls
                            let res = ctx
                                .req_ctx
                                .execute(req)
                                .await
                                .map_err(|e| EvaluationError::IOException(e.to_string()))?;
                            if ctx.req_ctx.server.enable_http_validation() {
                                req_template
                                    .endpoint
                                    .output
                                    .validate(&res.body)
                                    .map_err(EvaluationError::from)?;
                            }

                            // Insert into cache for future requests
                            if is_get {
                                ctx.req_ctx.cache.insert(url, res.clone());
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

#[derive(Clone, Debug)]
pub enum Expression {
    Unsafe(Operation),
}

#[derive(Clone, Debug)]
pub enum Context {
    Value,
    Path(Vec<String>),
}
