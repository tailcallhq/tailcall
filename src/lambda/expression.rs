use std::collections::hash_map::DefaultHasher;
use std::fmt::Debug;
use std::future::Future;
use std::hash::{Hash, Hasher};
use std::num::NonZeroU64;
use std::ops::Deref;
use std::pin::Pin;
use std::sync::Arc;

use anyhow::Result;
use async_graphql_value::ConstValue;
use reqwest::Request;
use serde_json::Value;
use thiserror::Error;

use super::ResolverContextLike;
use crate::config::group_by::GroupBy;
use crate::config::GraphQLOperationType;
use crate::data_loader::{DataLoader, Loader};
use crate::graphql::{self, GraphqlDataLoader};
use crate::grpc;
use crate::grpc::data_loader::GrpcDataLoader;
use crate::grpc::protobuf::ProtobufOperation;
use crate::grpc::request::execute_grpc_request;
use crate::grpc::request_template::RenderedRequestTemplate;
use crate::http::{self, cache_policy, DataLoaderRequest, HttpDataLoader, Response};
#[cfg(feature = "unsafe-js")]
use crate::javascript;
use crate::json::JsonLike;
use crate::lambda::EvaluationContext;

#[derive(Clone, Debug)]
pub enum Expression {
  Context(Context),
  Literal(Value), // TODO: this should async_graphql::Value
  EqualTo(Box<Expression>, Box<Expression>),
  Unsafe(Unsafe),
  Input(Box<Expression>, Vec<String>),
  Cache(Cache),
}

#[derive(Clone, Debug)]
pub enum Context {
  Value,
  Path(Vec<String>),
}

#[derive(Clone, Debug)]
pub enum Unsafe {
  Http {
    req_template: http::RequestTemplate,
    group_by: Option<GroupBy>,
    dl_id: Option<DataLoaderId>,
  },
  GraphQLEndpoint {
    req_template: graphql::RequestTemplate,
    field_name: String,
    batch: bool,
    dl_id: Option<DataLoaderId>,
  },
  Grpc {
    req_template: grpc::RequestTemplate,
    group_by: Option<GroupBy>,
    dl_id: Option<DataLoaderId>,
  },
  JS(Box<Expression>, String),
}

#[derive(Clone, Debug)]
pub struct Cache {
  hasher: DefaultHasher,
  max_age: NonZeroU64,
  source: Box<Expression>,
}

impl Cache {
  pub fn new(hasher: DefaultHasher, max_age: NonZeroU64, source: Box<Expression>) -> Self {
    Self { hasher, max_age, source }
  }

  pub fn hasher(&self) -> &DefaultHasher {
    &self.hasher
  }

  pub fn max_age(&self) -> NonZeroU64 {
    self.max_age
  }

  pub fn source(&self) -> &Expression {
    &self.source
  }
}

#[derive(Clone, Copy, Debug)]
pub struct DataLoaderId(pub usize);

#[derive(Debug, Error)]
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
          Unsafe::Http { req_template, dl_id, .. } => {
            let req = req_template.to_request(ctx)?;
            let is_get = req.method() == reqwest::Method::GET;

            let res = if is_get && ctx.req_ctx.is_batching_enabled() {
              let data_loader: Option<&DataLoader<DataLoaderRequest, HttpDataLoader>> =
                dl_id.and_then(|index| ctx.req_ctx.http_data_loaders.get(index.0));
              execute_request_with_dl(ctx, req, data_loader).await?
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
          Unsafe::GraphQLEndpoint { req_template, field_name, dl_id, .. } => {
            let req = req_template.to_request(ctx)?;

            let res = if ctx.req_ctx.upstream.batch.is_some()
              && matches!(req_template.operation_type, GraphQLOperationType::Query)
            {
              let data_loader: Option<&DataLoader<DataLoaderRequest, GraphqlDataLoader>> =
                dl_id.and_then(|index| ctx.req_ctx.gql_data_loaders.get(index.0));
              execute_request_with_dl(ctx, req, data_loader).await?
            } else {
              execute_raw_request(ctx, req).await?
            };

            set_cache_control(ctx, &res);
            parse_graphql_response(ctx, res, field_name)
          }
          Unsafe::Grpc { req_template, dl_id, .. } => {
            let rendered = req_template.render(ctx)?;

            let res = if ctx.req_ctx.upstream.batch.is_some()
              // TODO: share check for operation_type for resolvers
              && matches!(req_template.operation_type, GraphQLOperationType::Query)
            {
              let data_loader: Option<&DataLoader<grpc::DataLoaderRequest, GrpcDataLoader>> =
                dl_id.and_then(|index| ctx.req_ctx.grpc_data_loaders.get(index.0));
              execute_grpc_request_with_dl(ctx, rendered, data_loader).await?
            } else {
              let req = rendered.to_request()?;
              execute_raw_grpc_request(ctx, req, &req_template.operation).await?
            };

            set_cache_control(ctx, &res);

            Ok(res.body)
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
        Expression::Cache(cache) => {
          if let Some(key) = get_cache_key(ctx, cache.hasher.clone()) {
            if let Some(cache_value) = ctx.req_ctx.cache_get(&key) {
              Ok(cache_value.to_owned())
            } else {
              let result = cache.source.eval(ctx).await;
              if let Ok(val) = &result {
                ctx.req_ctx.cache_insert(key, val.clone(), cache.max_age);
              }
              result
            }
          } else {
            cache.source.eval(ctx).await
          }
        }
      }
    })
  }
}

pub fn hash_const_value<H: Hasher>(const_value: &ConstValue, state: &mut H) {
  match const_value {
    ConstValue::Null => {}
    ConstValue::Boolean(val) => val.hash(state),
    ConstValue::Enum(name) => name.hash(state),
    ConstValue::Number(num) => num.hash(state),
    ConstValue::Binary(bytes) => bytes.hash(state),
    ConstValue::String(string) => string.hash(state),
    ConstValue::List(list) => list.iter().for_each(|val| hash_const_value(val, state)),
    ConstValue::Object(object) => {
      let mut tmp_list: Vec<_> = object.iter().collect();
      tmp_list.sort_by(|(key1, _), (key2, _)| key1.cmp(key2));
      tmp_list.iter().for_each(|(key, value)| {
        key.hash(state);
        hash_const_value(value, state);
      })
    }
  }
}

fn get_cache_key<'a, H: Hasher + Clone>(
  ctx: &'a EvaluationContext<'a, impl ResolverContextLike<'a>>,
  mut hasher: H,
) -> Option<u64> {
  // Hash on parent value
  if let Some(const_value) = ctx
    .graphql_ctx
    .value()
    // TODO: handle _id, id, or any field that has @key on it.
    .filter(|value| value != &&ConstValue::Null)
    .map(|data| data.get_key("id"))
  {
    // Hash on parent's id only?
    hash_const_value(const_value?, &mut hasher)
  }

  let key = ctx.graphql_ctx.args().map(|value_map| {
    value_map
      .iter()
      .map(|(key, value)| {
        let mut hasher = hasher.clone();
        key.hash(&mut hasher);
        hash_const_value(value, &mut hasher);
        hasher.finish()
      })
      .fold(hasher.finish(), |acc, val| acc ^ val)
  });
  key
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
      .universal_http_client
      .execute(req)
      .await
      .map_err(|e| EvaluationError::IOException(e.to_string()))?,
  )
}

async fn execute_raw_grpc_request<'ctx, Ctx: ResolverContextLike<'ctx>>(
  ctx: &EvaluationContext<'ctx, Ctx>,
  req: Request,
  operation: &ProtobufOperation,
) -> Result<Response> {
  Ok(
    execute_grpc_request(ctx.req_ctx.http2_only_client.deref(), operation, req)
      .await
      .map_err(|e| EvaluationError::IOException(e.to_string()))?,
  )
}

async fn execute_grpc_request_with_dl<
  'ctx,
  Ctx: ResolverContextLike<'ctx>,
  Dl: Loader<grpc::DataLoaderRequest, Value = Response, Error = Arc<anyhow::Error>>,
>(
  ctx: &EvaluationContext<'ctx, Ctx>,
  rendered: RenderedRequestTemplate,
  data_loader: Option<&DataLoader<grpc::DataLoaderRequest, Dl>>,
) -> Result<Response> {
  let headers = ctx
    .req_ctx
    .upstream
    .batch
    .clone()
    .map(|s| s.headers)
    .unwrap_or_default();
  let endpoint_key = grpc::DataLoaderRequest::new(rendered, headers);

  Ok(
    data_loader
      .unwrap()
      .load_one(endpoint_key)
      .await
      .map_err(|e| EvaluationError::IOException(e.to_string()))?
      .unwrap_or_default(),
  )
}

async fn execute_request_with_dl<
  'ctx,
  Ctx: ResolverContextLike<'ctx>,
  Dl: Loader<DataLoaderRequest, Value = Response, Error = Arc<anyhow::Error>>,
>(
  ctx: &EvaluationContext<'ctx, Ctx>,
  req: Request,
  data_loader: Option<&DataLoader<DataLoaderRequest, Dl>>,
) -> Result<Response> {
  let headers = ctx
    .req_ctx
    .upstream
    .batch
    .clone()
    .map(|s| s.headers)
    .unwrap_or_default();
  let endpoint_key = crate::http::DataLoaderRequest::new(req, headers);

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
