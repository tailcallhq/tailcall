use std::collections::HashSet;
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use anyhow::Result;
use async_graphql_value::ConstValue;
use futures_util::future::join_all;
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
use crate::helpers::value::HashableConstValue;
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
  Logic(Logic),
  Relation(Relation),
  List(List),
  Math(Math),
  Concurrency(Concurrency, Box<Expression>),
}

#[derive(Clone, Debug)]
pub enum Concurrency {
  Parallel,
  Sequential,
}

#[derive(Clone, Debug)]
pub enum List {
  Concat(Vec<Expression>),
}

#[derive(Clone, Debug)]
pub enum Relation {
  Intersection(Vec<Expression>),
  Difference(Vec<Expression>, Vec<Expression>),
  Equals(Box<Expression>, Box<Expression>),
  Gt(Box<Expression>, Box<Expression>),
  Gte(Box<Expression>, Box<Expression>),
  Lt(Box<Expression>, Box<Expression>),
  Lte(Box<Expression>, Box<Expression>),
  Max(Vec<Expression>),
  Min(Vec<Expression>),
  PathEq(Box<Expression>, Vec<String>, Box<Expression>),
  PropEq(Box<Expression>, String, Box<Expression>),
  SortPath(Vec<Expression>, Vec<String>),
  SymmetricDifference(Vec<Expression>, Vec<Expression>),
  Union(Vec<Expression>, Vec<Expression>),
}

#[derive(Clone, Debug)]
pub enum Logic {
  If {
    cond: Box<Expression>,
    then: Box<Expression>,
    els: Box<Expression>,
  },
  AllPass(Vec<Expression>),
  And(Box<Expression>, Box<Expression>),
  AnyPass(Vec<Expression>),
  Cond(Box<Expression>, Vec<(Box<Expression>, Box<Expression>)>),
  DefaultTo(Box<Expression>, Box<Expression>),
  IsEmpty(Box<Expression>),
  Not(Box<Expression>),
  Or(Box<Expression>, Box<Expression>),
}

#[derive(Clone, Debug)]
pub enum Math {
  Mod(Box<Expression>, Box<Expression>),
  Add(Box<Expression>, Box<Expression>),
  Dec(Box<Expression>),
  Divide(Box<Expression>, Box<Expression>),
  Inc(Box<Expression>),
  Multiply(Box<Expression>, Box<Expression>),
  Negate(Box<Expression>),
  Product(Vec<Expression>),
  Subtract(Box<Expression>, Box<Expression>),
  Sum(Vec<Expression>),
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

  #[error("ConcatException: {0:?}")]
  ConcatException(String),

  #[error("IntersectionException: {0:?}")]
  IntersectionException(String),
}

impl<'a> From<crate::valid::ValidationError<&'a str>> for EvaluationError {
  fn from(_value: crate::valid::ValidationError<&'a str>) -> Self {
    EvaluationError::APIValidationError(_value.as_vec().iter().map(|e| e.message.to_owned()).collect())
  }
}

impl Expression {
  pub fn concurrency(self, conc: Concurrency) -> Self {
    Expression::Concurrency(conc, Box::new(self))
  }

  pub fn in_parallel(self) -> Self {
    self.concurrency(Concurrency::Parallel)
  }

  pub fn parallel_when(self, cond: bool) -> Self {
    if cond {
      self.concurrency(Concurrency::Parallel)
    } else {
      self
    }
  }

  pub fn in_sequence(self) -> Self {
    self.concurrency(Concurrency::Sequential)
  }

  pub fn eval<'a, Ctx: ResolverContextLike<'a> + Sync + Send>(
    &'a self,
    ctx: &'a EvaluationContext<'a, Ctx>,
    conc: &'a Concurrency,
  ) -> Pin<Box<dyn Future<Output = Result<async_graphql::Value>> + 'a + Send>> {
    Box::pin(async move {
      match self {
        Expression::Concurrency(conc, expr) => Ok(expr.eval(ctx, conc).await?),
        Expression::Context(op) => match op {
          Context::Value => Ok(ctx.value().cloned().unwrap_or(async_graphql::Value::Null)),
          Context::Path(path) => Ok(ctx.path_value(path).cloned().unwrap_or(async_graphql::Value::Null)),
        },
        Expression::Input(input, path) => {
          let inp = &input.eval(ctx, conc).await?;
          Ok(inp.get_path(path).unwrap_or(&async_graphql::Value::Null).clone())
        }
        Expression::Literal(value) => Ok(serde_json::from_value(value.clone())?),
        Expression::EqualTo(left, right) => Ok(async_graphql::Value::from(
          left.eval(ctx, conc).await? == right.eval(ctx, conc).await?,
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

            let res = if ctx.req_ctx.upstream.batch.is_some() &&
                // TODO: share check for operation_type for resolvers
                matches!(req_template.operation_type, GraphQLOperationType::Query)
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
              let input = input.eval(ctx, conc).await?;
              result = javascript::execute_js(script, input, Some(ctx.timeout))
                .map_err(|e| EvaluationError::JSException(e.to_string()).into());
            }
            result
          }
        },

        Expression::Relation(relation) => eval_relation(ctx, relation, conc).await,
        Expression::Logic(logic) => eval_logic(ctx, logic, conc).await,
        Expression::List(list) => eval_list(ctx, list, conc).await,
        Expression::Math(_) => todo!(),
      }
    })
  }
}

async fn eval_relation<'a, Ctx: ResolverContextLike<'a> + Sync + Send>(
  ctx: &'a EvaluationContext<'a, Ctx>,
  relation: &'a Relation,
  conc: &'a Concurrency,
) -> Result<async_graphql::Value> {
  match relation {
    Relation::Intersection(list) => {
      let results = join_all(list.iter().map(|expr| expr.eval(ctx, conc))).await;

      let mut results_iter = results.into_iter();

      let set: HashSet<_> = match results_iter.next() {
        Some(first) => match first? {
          ConstValue::List(list) => list.into_iter().map(HashableConstValue).collect(),
          _ => Err(EvaluationError::IntersectionException("element is not a list".into()))?,
        },
        None => Err(EvaluationError::IntersectionException("element is not a list".into()))?,
      };

      let final_set = results_iter.try_fold(set, |mut acc, result| match result? {
        ConstValue::List(list) => {
          let set: HashSet<_> = list.into_iter().map(HashableConstValue).collect();
          acc = acc.intersection(&set).cloned().collect();
          Ok::<_, anyhow::Error>(acc)
        }
        _ => Err(EvaluationError::IntersectionException("element is not a list".into()))?,
      })?;

      Ok(
        final_set
          .into_iter()
          .map(|HashableConstValue(const_value)| const_value)
          .collect(),
      )
    }
    Relation::Difference(_, _) => todo!(),
    Relation::Equals(_, _) => todo!(),
    Relation::Gt(_, _) => todo!(),
    Relation::Gte(_, _) => todo!(),
    Relation::Lt(_, _) => todo!(),
    Relation::Lte(_, _) => todo!(),
    Relation::Max(_) => todo!(),
    Relation::Min(_) => todo!(),
    Relation::PathEq(_, _, _) => todo!(),
    Relation::PropEq(_, _, _) => todo!(),
    Relation::SortPath(_, _) => todo!(),
    Relation::SymmetricDifference(_, _) => todo!(),
    Relation::Union(_, _) => todo!(),
  }
}

async fn eval_list<'a, Ctx: ResolverContextLike<'a> + Sync + Send>(
  ctx: &'a EvaluationContext<'a, Ctx>,
  list: &'a List,
  conc: &'a Concurrency,
) -> Result<async_graphql::Value> {
  match list {
    List::Concat(list) => join_all(list.iter().map(|expr| expr.eval(ctx, conc)))
      .await
      .into_iter()
      .try_fold(async_graphql::Value::List(vec![]), |acc, result| match (acc, result?) {
        (ConstValue::List(mut lhs), ConstValue::List(rhs)) => {
          lhs.extend(rhs.into_iter());
          Ok(ConstValue::List(lhs))
        }
        _ => Err(EvaluationError::ConcatException("element is not a list".into()))?,
      }),
  }
}

async fn eval_logic<'a, Ctx: ResolverContextLike<'a> + Sync + Send>(
  ctx: &'a EvaluationContext<'a, Ctx>,
  logic: &'a Logic,
  conc: &'a Concurrency,
) -> Result<async_graphql::Value> {
  Ok(match logic {
    Logic::And(lhs, rhs) => {
      let lhs_val = lhs.eval(ctx, conc).await?;
      (is_truthy(lhs_val) && is_truthy(rhs.eval(ctx, conc).await?)).into()
    }
    Logic::AnyPass(list) => {
      let futures = list.iter().map(|expr| expr.eval(ctx, conc));
      let init = false;
      let f = |acc, result| Ok::<_, anyhow::Error>(acc || is_truthy(result?));

      match *conc {
        Concurrency::Parallel => join_all(futures.into_iter()).await.into_iter().try_fold(init, f),
        Concurrency::Sequential => {
          let mut result = Ok(init);
          for future in futures.into_iter() {
            result = f(result?, future.await);
          }
          result
        }
      }?
      .into()
    }
    Logic::Cond(default, list) => match *conc {
      Concurrency::Sequential => {
        let mut result = None;
        for (cond, expr) in list.iter() {
          if is_truthy(cond.eval(ctx, conc).await?) {
            result = Some(expr.eval(ctx, conc).await?);
            break;
          }
        }
        result.unwrap_or(default.eval(ctx, conc).await?)
      }
      Concurrency::Parallel => {
        let true_cond_index = join_all(list.iter().map(|(cond, _)| cond.eval(ctx, conc)))
          .await
          .into_iter()
          .enumerate()
          .find_map(|(index, cond)| Some(is_truthy_ref(cond.as_ref().ok()?)).map(|_| index));

        if let Some(index) = true_cond_index {
          let (_, value) = list
            .get(index)
            .ok_or(anyhow::anyhow!("no condition found at index {index}"))?;
          value.eval(ctx, conc).await?
        } else {
          default.eval(ctx, conc).await?
        }
      }
    },
    Logic::DefaultTo(value, default) => {
      let result = value.eval(ctx, conc).await?;
      if is_empty(&result) {
        default.eval(ctx, conc).await?
      } else {
        result
      }
    }
    Logic::IsEmpty(expr) => is_empty(&expr.eval(ctx, conc).await?).into(),
    Logic::Not(expr) => (!is_truthy(expr.eval(ctx, conc).await?)).into(),
    Logic::Or(lhs, rhs) => (is_truthy(lhs.eval(ctx, conc).await?) || is_truthy(rhs.eval(ctx, conc).await?)).into(),

    Logic::AllPass(list) => {
      let futures = list.iter().map(|expr| expr.eval(ctx, conc));
      let init = true;
      let f = |acc, result| Ok::<_, anyhow::Error>(acc && is_truthy(result?));

      match *conc {
        Concurrency::Parallel => join_all(futures.into_iter()).await.into_iter().try_fold(init, f),
        Concurrency::Sequential => {
          let mut result = Ok(init);
          for future in futures.into_iter() {
            result = f(result?, future.await);
          }
          result
        }
      }?
      .into()
    }
    Logic::If { cond, then, els } => {
      let cond = cond.eval(ctx, conc).await?;
      if is_truthy(cond) {
        then.eval(ctx, conc).await?
      } else {
        els.eval(ctx, conc).await?
      }
    }
  })
}

/// Check if a value is truthy
///
/// Special cases:
/// 1. An empty string is considered falsy
/// 2. A collection of bytes is truthy, even if the value in those bytes is 0. An empty collection is falsy.
fn is_truthy(value: async_graphql::Value) -> bool {
  use async_graphql::{Number, Value};
  use hyper::body::Bytes;

  match value {
    Value::Null => false,
    Value::Enum(_) => true,
    Value::List(_) => true,
    Value::Object(_) => true,
    Value::String(s) => !s.is_empty(),
    Value::Boolean(b) => b,
    Value::Number(n) => n != Number::from(0),
    Value::Binary(b) => b != Bytes::default(),
  }
}

fn is_truthy_ref(value: &async_graphql::Value) -> bool {
  use async_graphql::{Number, Value};
  use hyper::body::Bytes;

  match value {
    &Value::Null => false,
    &Value::Enum(_) => true,
    &Value::List(_) => true,
    &Value::Object(_) => true,
    Value::String(s) => !s.is_empty(),
    &Value::Boolean(b) => b,
    Value::Number(n) => n != &Number::from(0),
    Value::Binary(b) => b != &Bytes::default(),
  }
}

fn is_empty(value: &async_graphql::Value) -> bool {
  match value {
    ConstValue::Null => true,
    ConstValue::Number(_) | ConstValue::Boolean(_) | ConstValue::Enum(_) => false,
    ConstValue::Binary(bytes) => bytes.is_empty(),
    ConstValue::List(list) => list.is_empty(),
    ConstValue::Object(obj) => obj.is_empty(),
    ConstValue::String(string) => string.is_empty(),
  }
}

fn set_cache_control<'ctx, Ctx: ResolverContextLike<'ctx>>(
  ctx: &EvaluationContext<'ctx, Ctx>,
  res: &Response<async_graphql::Value>,
) {
  if ctx.req_ctx.server.get_enable_cache_control() && res.status.is_success() {
    if let Some(policy) = cache_policy(res) {
      ctx.req_ctx.set_cache_control(policy);
    }
  }
}

async fn execute_raw_request<'ctx, Ctx: ResolverContextLike<'ctx>>(
  ctx: &EvaluationContext<'ctx, Ctx>,
  req: Request,
) -> Result<Response<async_graphql::Value>> {
  ctx
    .req_ctx
    .h_client
    .execute(req)
    .await
    .map_err(|e| EvaluationError::IOException(e.to_string()))?
    .to_json()
}

async fn execute_raw_grpc_request<'ctx, Ctx: ResolverContextLike<'ctx>>(
  ctx: &EvaluationContext<'ctx, Ctx>,
  req: Request,
  operation: &ProtobufOperation,
) -> Result<Response<async_graphql::Value>> {
  Ok(
    execute_grpc_request(&ctx.req_ctx.h2_client, operation, req)
      .await
      .map_err(|e| EvaluationError::IOException(e.to_string()))?,
  )
}

async fn execute_grpc_request_with_dl<
  'ctx,
  Ctx: ResolverContextLike<'ctx>,
  Dl: Loader<grpc::DataLoaderRequest, Value = Response<async_graphql::Value>, Error = Arc<anyhow::Error>>,
>(
  ctx: &EvaluationContext<'ctx, Ctx>,
  rendered: RenderedRequestTemplate,
  data_loader: Option<&DataLoader<grpc::DataLoaderRequest, Dl>>,
) -> Result<Response<async_graphql::Value>> {
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
  Dl: Loader<DataLoaderRequest, Value = Response<async_graphql::Value>, Error = Arc<anyhow::Error>>,
>(
  ctx: &EvaluationContext<'ctx, Ctx>,
  req: Request,
  data_loader: Option<&DataLoader<DataLoaderRequest, Dl>>,
) -> Result<Response<async_graphql::Value>> {
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
  res: Response<async_graphql::Value>,
  field_name: &str,
) -> Result<async_graphql::Value> {
  let res: async_graphql::Response = serde_json::from_value(res.body.into_json()?)?;

  for error in res.errors {
    ctx.add_error(error);
  }

  Ok(res.data.get_key(field_name).map(|v| v.to_owned()).unwrap_or_default())
}

#[cfg(test)]
mod tests {
  use async_graphql::{Name, Number, Value};
  use hyper::body::Bytes;
  use indexmap::IndexMap;

  use super::is_truthy;

  #[test]
  fn test_is_truthy() {
    assert!(is_truthy(Value::Enum(Name::new("EXAMPLE"))));
    assert!(is_truthy(Value::List(vec![])));
    assert!(is_truthy(Value::Object(IndexMap::default())));
    assert!(is_truthy(Value::String("Hello".to_string())));
    assert!(is_truthy(Value::Boolean(true)));
    assert!(is_truthy(Value::Number(Number::from(1))));
    assert!(is_truthy(Value::Binary(Bytes::from_static(&[0, 1, 2]))));

    assert!(!is_truthy(Value::Null));
    assert!(!is_truthy(Value::String("".to_string())));
    assert!(!is_truthy(Value::Boolean(false)));
    assert!(!is_truthy(Value::Number(Number::from(0))));
    assert!(!is_truthy(Value::Binary(Bytes::default())));
  }
}
