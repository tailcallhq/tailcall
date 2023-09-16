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
use async_graphql_value::ConstValue;
use std::path::PathBuf;
use wasmer::{imports, Instance, Module, Store};

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

#[derive(Clone, Debug)]
pub enum Operation {
  Endpoint(RequestTemplate),
  JS(Box<Expression>, String),
  WasmPlugin(Box<Expression>, String),
}

#[derive(Debug, Error, Serialize)]
pub enum EvaluationError {
  #[error("IOException: {0}")]
  IOException(String),

  #[error("JSException: {0}")]
  JSException(String),

  #[error("APIValidationError: {0:?}")]
  APIValidationError(Vec<String>),

  #[error("WasmPluginException: {0}")]
  WasmPluginException(String),
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
            Operation::WasmPlugin(input, name) => {
                    println!("WasmPlugin: {}", name);
                    let mut dir_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
                    let file_path = format!("wasmplugins/{}.wat", name);
                    dir_path.push(file_path);
                    println!("file_path: {:?}", dir_path);
                    let file = std::fs::read_to_string(dir_path.clone())?;
                    let module_wat = file.as_str();

                    let mut store = Store::default();
                    let module = Module::new(&store, module_wat)?;
                    // The module doesn't import anything, so we create an empty import object.
                    let import_object = imports! {};
                    let instance = Instance::new(&mut store, &module, &import_object)?;

                    let add_one = instance.exports.get_function("add_one")?;
                    let inp = &input.eval(ctx).await?;
                    let id = inp.get_path(&["id".to_string()]).unwrap();
                    match id {
                        ConstValue::Number(n) => {
                            let result =
                                add_one.call(&mut store, &[wasmer::Value::I32(n.as_i64().unwrap() as i32)])?;
                            match *result {
                                [wasmer::Value::I32(x)] => Ok(async_graphql::Value::from(x)),
                                _ => panic!("unexpected type returned from add_one"),
                            }
                        }
                        _ => panic!("unexpected type returned from input"),
                    }
                }
            }
        }
      }
    })
  }
}
