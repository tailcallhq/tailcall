use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use anyhow::Result;
use async_graphql::dataloader::{DataLoader, Loader, NoCache};
use reqwest::Request;
use serde::Serialize;
use serde_json::Value;
use thiserror::Error;

use super::ResolverContextLike;
use crate::config::group_by::GroupBy;
use crate::config::GraphQLOperationType;
use crate::graphql_request_template::GraphqlRequestTemplate;
use crate::http::{
  cache_policy, DataLoaderRequest, GetDataLoader, GraphqlDataLoader, HttpDataLoader, RequestContext, Response,
};
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
  Unsafe(Unsafe),
  Input(Box<Expression>, Vec<String>),
}

#[derive(Clone, Debug)]
pub enum Context {
  Value,
  Path(Vec<String>),
}

#[derive(Clone)]
pub enum Unsafe {
  Http(RequestTemplate, Option<GroupBy>, Option<usize>),
  GraphQLEndpoint {
    req_template: GraphqlRequestTemplate,
    field_name: String,
    batch: bool,
    data_loader_index: Option<usize>,
  },
  JS(Box<Expression>, String),
}

impl Debug for Unsafe {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Unsafe::Http(req_template, group_by, dl) => f
        .debug_struct("Http")
        .field("req_template", req_template)
        .field("group_by", group_by)
        .field("dl", dl)
        .finish(),
      Unsafe::GraphQLEndpoint { req_template, field_name, batch, .. } => f
        .debug_struct("GraphQLEndpoint")
        .field("req_template", req_template)
        .field("field_name", field_name)
        .field("batch", batch)
        .finish(),
      Unsafe::JS(input, script) => f
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
        Expression::Unsafe(operation) => match operation {
          Unsafe::Http(req_template, _, data_loader_index) => {
            let req = req_template.to_request(ctx)?;
            let is_get = req.method() == reqwest::Method::GET;

            let res = if is_get && ctx.req_ctx.upstream.batch.is_some() {
              execute_request_with_dl::<_, HttpDataLoader>(ctx, req, data_loader_index).await?
            } else {
              execute_raw_request(ctx, req).await?
            };

            if ctx.req_ctx.server.get_enable_http_validation() {
              req_template
                .endpoint
                .output
                .validate(&res.body)
                .to_result()
                .map_err(EvaluationError::from)?;
            }

            set_cache_control(ctx, &res);

            Ok(res.body)
          }
          Unsafe::GraphQLEndpoint { req_template, field_name, data_loader_index, .. } => {
            let req = req_template.to_request(ctx)?;

            let res = if ctx.req_ctx.upstream.batch.is_some()
              && matches!(req_template.operation_type, GraphQLOperationType::Query)
            {
              execute_request_with_dl::<_, GraphqlDataLoader>(ctx, req, data_loader_index).await?
            } else {
              execute_raw_request(ctx, req).await?
            };

            set_cache_control(ctx, &res);
            parse_graphql_response(ctx, res, field_name)
          }
          Unsafe::JS(input, script) => {
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
        },
      }
    })
  }
}

fn set_cache_control<'ctx, Ctx: ResolverContextLike<'ctx>>(ctx: &EvaluationContext<'ctx, Ctx>, res: &Response) {
  if ctx.req_ctx.server.get_enable_cache_control() && res.status.is_success() {
    if let Some(policy) = cache_policy(res) {
      ctx.req_ctx.set_cache_control(policy);
    }
  }
}

async fn execute_raw_request<'ctx, Ctx: ResolverContextLike<'ctx>>(
  ctx: &EvaluationContext<'ctx, Ctx>,
  req: Request,
) -> Result<Response> {
  Ok(
    ctx
      .req_ctx
      .execute(req)
      .await
      .map_err(|e| EvaluationError::IOException(e.to_string()))?,
  )
}

async fn execute_request_with_dl<
  'ctx,
  Ctx: ResolverContextLike<'ctx>,
  Dl: Loader<DataLoaderRequest, Value = Response, Error = Arc<anyhow::Error>>,
>(
  ctx: &EvaluationContext<'ctx, Ctx>,
  req: Request,
  data_loader_index: &Option<usize>,
) -> Result<Response>
where
  RequestContext: GetDataLoader<Dl>,
{
  let headers = ctx
    .req_ctx
    .upstream
    .batch
    .clone()
    .map(|s| s.headers)
    .unwrap_or_default();
  let endpoint_key = crate::http::DataLoaderRequest::new(req, headers);

  let data_loader: Option<&DataLoader<Dl>> = data_loader_index.and_then(|index| ctx.req_ctx.get_data_loader(index));

  Ok(
    data_loader
      .unwrap()
      .load_one(endpoint_key)
      .await
      .map_err(|e| EvaluationError::IOException(e.to_string()))?
      .unwrap_or_default(),
  )
}

fn parse_graphql_response<'ctx, Ctx: ResolverContextLike<'ctx>>(
  ctx: &EvaluationContext<'ctx, Ctx>,
  res: Response,
  field_name: &str,
) -> Result<async_graphql::Value> {
  let res: async_graphql::Response = serde_json::from_value(res.body.into_json()?)?;

  for error in res.errors {
    ctx.add_error(error);
  }

  Ok(res.data.get_key(field_name).map(|v| v.to_owned()).unwrap_or_default())
}
