use std::any::Any;

use anyhow::Result;
use async_graphql::{BatchResponse, Executor, Value};
use hyper::header::{HeaderValue, CACHE_CONTROL, CONTENT_TYPE};
use hyper::{Body, Response, StatusCode};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

#[async_trait::async_trait]
pub trait GraphQLRequestLike {
    fn data<D: Any + Clone + Send + Sync>(self, data: D) -> Self;
    async fn execute<E>(self, executor: &E) -> GraphQLResponse
    where
        E: Executor;
}

#[derive(Debug, Deserialize)]
pub struct GraphQLBatchRequest(pub async_graphql::BatchRequest);
impl GraphQLBatchRequest {}

#[async_trait::async_trait]
impl GraphQLRequestLike for GraphQLBatchRequest {
    fn data<D: Any + Clone + Send + Sync>(mut self, data: D) -> Self {
        for request in self.0.iter_mut() {
            request.data.insert(data.clone());
        }
        self
    }
    /// Shortcut method to execute the request on the executor.
    async fn execute<E>(self, executor: &E) -> GraphQLResponse
    where
        E: Executor,
    {
        GraphQLResponse(executor.execute_batch(self.0).await)
    }
}

#[derive(Debug, Deserialize)]
pub struct GraphQLRequest(pub async_graphql::Request);

impl GraphQLRequest {}

#[async_trait::async_trait]
impl GraphQLRequestLike for GraphQLRequest {
    #[must_use]
    fn data<D: Any + Send + Sync>(mut self, data: D) -> Self {
        self.0.data.insert(data);
        self
    }
    /// Shortcut method to execute the request on the schema.
    async fn execute<E>(self, executor: &E) -> GraphQLResponse
    where
        E: Executor,
    {
        GraphQLResponse(executor.execute(self.0).await.into())
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

static APPLICATION_JSON: Lazy<HeaderValue> =
    Lazy::new(|| HeaderValue::from_static("application/json"));

impl GraphQLResponse {
    fn build_response(&self, status: StatusCode, body: Body) -> Result<Response<Body>> {
        let mut response = Response::builder()
            .status(status)
            .header(CONTENT_TYPE, APPLICATION_JSON.as_ref())
            .body(body)?;

        if self.0.is_ok() {
            if let Some(cache_control) = self.0.cache_control().value() {
                response.headers_mut().insert(
                    CACHE_CONTROL,
                    HeaderValue::from_str(cache_control.as_str())?,
                );
            }
        }

        Ok(response)
    }

    fn default_body(&self) -> Result<Body> {
        Ok(Body::from(serde_json::to_string(&self.0)?))
    }

    pub fn to_response(self) -> Result<Response<hyper::Body>> {
        self.build_response(StatusCode::OK, self.default_body()?)
    }

    fn flatten_response(data: Value) -> Value {
        if let Value::Object(map) = data.clone() {
            if map.len() == 1 {
                if let Some(value) = map.values().next() {
                    return value.clone();
                }
            }
        }
        data
    }

    /// Transforms a plain `GraphQLResponse` into a `Response<Body>`.
    /// Differs as `to_response` by flattening the response's data
    /// `{"data": {"user": {"name": "John"}}}` becomes `{"name": "John"}`.
    pub fn to_rest_response(self) -> Result<Response<hyper::Body>> {
        if !self.0.is_ok() {
            return self.build_response(StatusCode::INTERNAL_SERVER_ERROR, self.default_body()?);
        }

        match self.0 {
            BatchResponse::Single(ref res) => {
                let item = Self::flatten_response(res.data.clone());
                let data = serde_json::to_string(&item)?;

                return self.build_response(StatusCode::OK, Body::from(data));
            }
            BatchResponse::Batch(ref list) => {
                let item = list
                    .iter()
                    .map(|res| Self::flatten_response(res.data.clone()))
                    .collect::<Value>();
                let data = serde_json::to_string(&item)?;

                return self.build_response(StatusCode::OK, Body::from(data));
            }
        }
    }

    /// Sets the `cache_control` for a given `GraphQLResponse`.
    ///
    /// The function modifies the `GraphQLResponse` to set the `cache_control`
    /// `max_age` to the specified `min_cache` value and `public` flag to
    /// `cache_public`
    ///
    /// # Arguments
    ///
    /// * `res` - The GraphQL response whose `cache_control` is to be set.
    /// * `min_cache` - The `max_age` value to be set for `cache_control`.
    /// * `cache_public` - The negation of `public` flag to be set for
    ///   `cache_control`.
    ///
    /// # Returns
    ///
    /// * A modified `GraphQLResponse` with updated `cache_control` `max_age`
    ///   and `public` flag.
    pub fn set_cache_control(mut self, min_cache: i32, cache_public: bool) -> GraphQLResponse {
        match self.0 {
            BatchResponse::Single(ref mut res) => {
                res.cache_control.max_age = min_cache;
                res.cache_control.public = cache_public;
            }
            BatchResponse::Batch(ref mut list) => {
                for res in list {
                    res.cache_control.max_age = min_cache;
                    res.cache_control.public = cache_public;
                }
            }
        };
        self
    }
}
