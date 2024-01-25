use std::marker::PhantomData;

use super::expression::{Context, Expression};
use super::{expression, IO};
use crate::{graphql, grpc, http};

#[derive(Clone)]
pub struct Lambda<A> {
  _output: PhantomData<fn() -> A>,
  pub expression: Expression,
}

impl<A> Lambda<A> {
  fn box_expr(self) -> Box<Expression> {
    Box::new(self.expression)
  }
  pub fn new(expression: Expression) -> Self {
    Self { _output: PhantomData, expression }
  }

  pub fn eq(self, other: Self) -> Lambda<bool> {
    Lambda::new(Expression::EqualTo(self.box_expr(), Box::new(other.expression)))
  }

  pub fn to_js(self, script: String) -> Lambda<serde_json::Value> {
    Lambda::new(Expression::IO(IO::JS(self.box_expr(), script)))
  }

  pub fn to_input_path(self, path: Vec<String>) -> Lambda<serde_json::Value> {
    Lambda::new(Expression::Input(self.box_expr(), path))
  }
}

impl Lambda<serde_json::Value> {
  pub fn context() -> Self {
    Lambda::new(Expression::Context(expression::Context::Value))
  }

  pub fn context_field(name: String) -> Self {
    Lambda::new(Expression::Context(Context::Path(vec![name])))
  }

  pub fn context_path(path: Vec<String>) -> Self {
    Lambda::new(Expression::Context(Context::Path(path)))
  }

  pub fn from_request_template(req_template: http::RequestTemplate) -> Lambda<serde_json::Value> {
    Lambda::new(Expression::IO(IO::Http { req_template, group_by: None, dl_id: None }))
  }

  pub fn from_graphql_request_template(
    req_template: graphql::RequestTemplate,
    field_name: String,
    batch: bool,
  ) -> Lambda<serde_json::Value> {
    Lambda::new(Expression::IO(IO::GraphQLEndpoint {
      req_template,
      field_name,
      batch,
      dl_id: None,
    }))
  }

  pub fn from_grpc_request_template(req_template: grpc::RequestTemplate) -> Lambda<serde_json::Value> {
    Lambda::new(Expression::IO(IO::Grpc { req_template, group_by: None, dl_id: None }))
  }
}

impl<A> From<A> for Lambda<A>
where
  serde_json::Value: From<A>,
{
  fn from(value: A) -> Self {
    let json = serde_json::Value::from(value);
    Lambda::new(Expression::Literal(json))
  }
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;
  use std::num::NonZeroU64;
  use std::sync::{Arc, Mutex, RwLock};
  use std::time::Duration;

  use anyhow::{anyhow, Result};
  use async_graphql_value::ConstValue;
  use async_trait::async_trait;
  use httpmock::Method::GET;
  use httpmock::MockServer;
  use hyper::body::Bytes;
  use hyper::HeaderMap;
  use reqwest::{Client, Request};
  use serde::de::DeserializeOwned;
  use serde_json::json;
  use ttl_cache::TtlCache;

  use crate::blueprint::Server;
  use crate::config::Config;
  use crate::endpoint::Endpoint;
  use crate::http::{RequestContext, RequestTemplate, Response};
  use crate::lambda::{Concurrent, EmptyResolverContext, Eval, EvaluationContext, Lambda};
  use crate::{Cache, EnvIO, HttpIO};

  fn get_req_ctx() -> RequestContext {
    let Config { server, upstream, .. } = Config::default();
    //TODO: default is used only in tests. Drop default and move it to test.
    let server = Server::try_from(server).unwrap();
    let test_http = Arc::new(TestHttpIO::init());
    let h_client = test_http.clone();
    let h2_client = test_http;
    RequestContext {
      req_headers: HeaderMap::new(),
      h_client,
      h2_client,
      server,
      upstream,
      http_data_loaders: Arc::new(vec![]),
      gql_data_loaders: Arc::new(vec![]),
      cache: Arc::new(TestCache::new()),
      grpc_data_loaders: Arc::new(vec![]),
      min_max_age: Arc::new(Mutex::new(None)),
      cache_public: Arc::new(Mutex::new(None)),
      env_vars: Arc::new(TestEnv::new()),
    }
  }

  struct TestEnv {
    vars: HashMap<String, String>,
  }
  impl TestEnv {
    fn new() -> Self {
      Self { vars: std::env::vars().collect() }
    }
  }
  impl EnvIO for TestEnv {
    fn get(&self, key: &str) -> Option<String> {
      self.vars.get(key).cloned()
    }
  }

  struct TestCache {
    data: Arc<RwLock<TtlCache<u64, ConstValue>>>,
  }

  impl TestCache {
    fn new() -> Self {
      Self { data: Arc::new(RwLock::new(TtlCache::new(100))) }
    }
  }

  #[async_trait]
  impl Cache for TestCache {
    type Key = u64;
    type Value = ConstValue;

    #[allow(clippy::too_many_arguments)]
    async fn set<'a>(&'a self, key: Self::Key, value: Self::Value, ttl: NonZeroU64) -> anyhow::Result<Self::Value> {
      let ttl = Duration::from_millis(ttl.get());
      self
        .data
        .write()
        .unwrap()
        .insert(key, value, ttl)
        .ok_or(anyhow!("unable to insert value"))
    }

    async fn get<'a>(&'a self, key: &'a Self::Key) -> anyhow::Result<Self::Value> {
      self
        .data
        .read()
        .unwrap()
        .get(key)
        .cloned()
        .ok_or(anyhow!("key not found"))
    }
  }

  #[derive(Default)]
  struct TestHttpIO {
    client: Client,
  }
  impl TestHttpIO {
    fn init() -> Self {
      Default::default()
    }
  }
  #[async_trait]
  impl HttpIO for TestHttpIO {
    async fn execute(&self, request: Request) -> anyhow::Result<Response<Bytes>> {
      let resp = self.client.execute(request).await?;
      let resp = Response::from_reqwest(resp).await?;
      Ok(resp)
    }
  }

  impl<B> Lambda<B>
  where
    B: DeserializeOwned,
  {
    async fn eval(self) -> Result<B> {
      let req_ctx = get_req_ctx();
      let ctx = EvaluationContext::new(&req_ctx, &EmptyResolverContext);
      let result = self.expression.eval(&ctx, &Concurrent::Sequential).await?;
      let json = serde_json::to_value(result)?;
      Ok(serde_json::from_value(json)?)
    }
  }

  #[tokio::test]
  async fn test_equal_to_true() {
    let lambda = Lambda::from(1.0).eq(Lambda::from(1.0));
    let result = lambda.eval().await.unwrap();
    assert!(result)
  }

  #[tokio::test]
  async fn test_equal_to_false() {
    let lambda = Lambda::from(1.0).eq(Lambda::from(2.0));
    let result = lambda.eval().await.unwrap();
    assert!(!result)
  }

  #[tokio::test]
  async fn test_endpoint() {
    let server = MockServer::start();

    server.mock(|when, then| {
      when.method(GET).path("/users");
      then
        .status(200)
        .header("content-type", "application/json")
        .json_body(json!({ "name": "Hans" }));
    });

    let endpoint = RequestTemplate::try_from(Endpoint::new(server.url("/users").to_string())).unwrap();
    let result = Lambda::from_request_template(endpoint).eval().await.unwrap();

    assert_eq!(result.as_object().unwrap().get("name").unwrap(), "Hans")
  }

  #[cfg(feature = "unsafe-js")]
  #[tokio::test]
  async fn test_js() {
    let result = Lambda::from(1.0).to_js("ctx + 100".to_string()).eval().await;
    let f64 = result.unwrap().as_f64().unwrap();
    assert_eq!(f64, 101.0)
  }
}
