use std::any::Any;

use anyhow::Result;
use async_graphql::Executor;
use hyper::header::{HeaderValue, CONTENT_TYPE};
use hyper::{Body, Response};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct GraphQLBatchRequest(pub async_graphql::BatchRequest);
impl GraphQLBatchRequest {
  /// Shortcut method to execute the request on the executor.
  pub async fn execute<E>(self, executor: &E) -> GraphQLResponse
  where
    E: Executor,
  {
    GraphQLResponse(executor.execute_batch(self.0).await)
  }
}
#[derive(Debug, Deserialize)]
pub struct GraphQLRequest(pub async_graphql::Request);

impl GraphQLRequest {
  /// Shortcut method to execute the request on the schema.
  pub async fn execute<E>(self, executor: &E) -> GraphQLResponse
  where
    E: Executor,
  {
    GraphQLResponse(executor.execute(self.0).await.into())
  }

  /// Insert some data for this request.
  #[must_use]
  pub fn data<D: Any + Send + Sync>(mut self, data: D) -> Self {
    self.0.data.insert(data);
    self
  }
}
#[derive(Debug, Serialize)]
pub struct GraphQLResponse(pub async_graphql::BatchResponse);
impl From<async_graphql::BatchResponse> for GraphQLResponse {
  fn from(batch: async_graphql::BatchResponse) -> Self {
    Self(batch)
  }
}
impl From<async_graphql::Response> for GraphQLResponse {
  fn from(res: async_graphql::Response) -> Self {
    Self(res.into())
  }
}

impl From<GraphQLQuery> for GraphQLRequest {
  fn from(query: GraphQLQuery) -> Self {
    let mut request = async_graphql::Request::new(query.query);

    if let Some(operation_name) = query.operation_name {
      request = request.operation_name(operation_name);
    }

    if let Some(variables) = query.variables {
      let value = serde_json::from_str(&variables).unwrap_or_default();
      let variables = async_graphql::Variables::from_json(value);
      request = request.variables(variables);
    }

    GraphQLRequest(request)
  }
}

#[derive(Debug)]
pub struct GraphQLQuery {
  query: String,
  operation_name: Option<String>,
  variables: Option<String>,
}

impl GraphQLQuery {
  /// Shortcut method to execute the request on the schema.
  pub async fn execute<E>(self, executor: &E) -> GraphQLResponse
  where
    E: Executor,
  {
    let request: GraphQLRequest = self.into();
    request.execute(executor).await
  }
}

impl GraphQLResponse {
  pub fn into_hyper_response(self) -> Result<Response<hyper::Body>> {
    let body = serde_json::to_string(&self.0)?;
    let mut response = Response::builder().header(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    if self.0.is_ok() {
      if let Some(cache_control) = self.0.cache_control().value() {
        response = response.header("cache-control", cache_control);
      }
    }

    for (name, value) in self.0.http_headers_iter() {
      if let Ok(value) = value.to_str() {
        response = response.header(name.as_str(), value);
      }
    }

    Ok(response.body(Body::from(body))?)
  }
}
