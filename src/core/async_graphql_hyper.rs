use std::any::Any;

use anyhow::Result;
use async_graphql::parser::types::ExecutableDocument;
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

    fn parse_query(&mut self) -> Option<&ExecutableDocument>;
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

    fn parse_query(&mut self) -> Option<&ExecutableDocument> {
        None
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

    fn parse_query(&mut self) -> Option<&ExecutableDocument> {
        self.0.parsed_query().ok()
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

    pub fn into_response(self) -> Result<Response<hyper::Body>> {
        self.build_response(StatusCode::OK, self.default_body()?)
    }

    fn flatten_response(data: &Value) -> &Value {
        match data {
            Value::Object(map) if map.len() == 1 => map.iter().next().unwrap().1,
            data => data,
        }
    }

    /// Transforms a plain `GraphQLResponse` into a `Response<Body>`.
    /// Differs as `to_response` by flattening the response's data
    /// `{"data": {"user": {"name": "John"}}}` becomes `{"name": "John"}`.
    pub fn into_rest_response(self) -> Result<Response<hyper::Body>> {
        if !self.0.is_ok() {
            return self.build_response(StatusCode::INTERNAL_SERVER_ERROR, self.default_body()?);
        }

        match self.0 {
            BatchResponse::Single(ref res) => {
                let item = Self::flatten_response(&res.data);
                let data = serde_json::to_string(item)?;

                self.build_response(StatusCode::OK, Body::from(data))
            }
            BatchResponse::Batch(ref list) => {
                let item = list
                    .iter()
                    .map(|res| Self::flatten_response(&res.data))
                    .collect::<Vec<&Value>>();
                let data = serde_json::to_string(&item)?;

                self.build_response(StatusCode::OK, Body::from(data))
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

#[cfg(test)]
mod tests {
    use async_graphql::{Name, Response, ServerError, Value};
    use hyper::StatusCode;
    use indexmap::IndexMap;
    use serde_json::json;

    use super::*;

    #[tokio::test]
    async fn test_to_rest_response_single() {
        let name = "John";

        let user = IndexMap::from([(Name::new("name"), Value::String(name.to_string()))]);
        let data = IndexMap::from([(Name::new("user"), Value::Object(user))]);

        let response = GraphQLResponse(BatchResponse::Single(Response::new(Value::Object(data))));
        let rest_response = response.into_rest_response().unwrap();

        assert_eq!(rest_response.status(), StatusCode::OK);
        assert_eq!(rest_response.headers()["content-type"], "application/json");
        assert_eq!(
            hyper::body::to_bytes(rest_response.into_body())
                .await
                .unwrap()
                .to_vec(),
            json!({ "name": name }).to_string().as_bytes().to_vec()
        );
    }

    #[tokio::test]
    async fn test_to_rest_response_batch() {
        let names = ["John", "Doe", "Jane"];

        let list = names
            .iter()
            .map(|name| {
                let user = IndexMap::from([(Name::new("name"), Value::String(name.to_string()))]);
                let data = IndexMap::from([(Name::new("user"), Value::Object(user))]);
                Response::new(Value::Object(data))
            })
            .collect();

        let response = GraphQLResponse(BatchResponse::Batch(list));
        let rest_response = response.into_rest_response().unwrap();

        assert_eq!(rest_response.status(), StatusCode::OK);
        assert_eq!(rest_response.headers()["content-type"], "application/json");
        assert_eq!(
            hyper::body::to_bytes(rest_response.into_body())
                .await
                .unwrap()
                .to_vec(),
            json!([
                { "name": names[0] },
                { "name": names[1] },
                { "name": names[2] }
            ])
            .to_string()
            .as_bytes()
            .to_vec()
        );
    }

    #[tokio::test]
    async fn test_to_rest_response_with_error() {
        let errors = ["Some error", "Another error"];
        let mut response: Response = Default::default();
        response.errors = errors
            .iter()
            .map(|error| ServerError::new(error.to_string(), None))
            .collect();
        let response = GraphQLResponse(BatchResponse::Single(response));
        let rest_response = response.into_rest_response().unwrap();

        assert_eq!(rest_response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(rest_response.headers()["content-type"], "application/json");
        assert_eq!(
            hyper::body::to_bytes(rest_response.into_body())
                .await
                .unwrap()
                .to_vec(),
            json!({
                "data": null,
                "errors": errors.iter().map(|error| {
                    json!({
                        "message": error,
                    })
                }).collect::<Vec<_>>()
            })
            .to_string()
            .as_bytes()
            .to_vec()
        );
    }
}
