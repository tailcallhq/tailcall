use std::collections::BTreeMap;
use std::fs;
use std::str::FromStr;
use std::sync::Arc;

use async_graphql::InputType;
use http_cache_semantics::RequestLike;
use hyper::{Body, Request};
use reqwest::header::{HeaderName, HeaderValue};
use serde::Deserialize;
use tailcall::blueprint::Blueprint;
use tailcall::config::Config;
use tailcall::http::{graphql_request, HttpClient, Method, Response, ServerContext};
use url::Url;

#[derive(Deserialize, Clone, Debug)]
pub struct APIRequest {
  method: Method,
  pub url: Url,
  pub headers: Option<BTreeMap<String, String>>,
  pub body: Option<serde_json::Value>,
}
#[derive(Deserialize, Clone, Debug)]
pub struct APIResponse {
  pub status: u16,
  pub headers: Option<BTreeMap<String, String>>,
  pub body: Option<serde_json::Value>,
}
#[derive(Deserialize, Clone, Debug)]
pub struct UpstreamRequest(pub APIRequest);
#[derive(Deserialize, Clone, Debug)]
pub struct UpstreamResponse(APIResponse);
#[derive(Deserialize)]
pub struct DownstreamRequest(pub APIRequest);
#[derive(Deserialize)]
pub struct DownstreamResponse(pub APIResponse);
#[derive(Deserialize)]
pub struct DownstreamAssertion {
  pub request: DownstreamRequest,
  pub response: DownstreamResponse,
}

#[derive(Default, Deserialize)]
pub struct APISpecification {
  pub config: String,
  pub name: String,
  pub description: Option<String>,
  pub upstream_mocks: Vec<(UpstreamRequest, UpstreamResponse)>,
  pub expected_upstream_requests: Vec<UpstreamRequest>,
  pub downstream_assertions: Vec<DownstreamAssertion>,
}

async fn read_config_from_path(config_path: &str) -> Option<Config> {
  Config::from_file_paths([config_path.to_string()].iter()).await.ok()
}

impl APISpecification {
  fn read(spec: &str) -> Option<Self> {
    let contents = fs::read_to_string(spec).ok()?;
    let spec = serde_json::from_str(&contents);

    spec.ok()
  }
  async fn status(&self, query: String) {
    let state = self.setup().await;
    let req = Request::builder()
      .method(Method::POST)
      .uri("http://localhost:8080/graphql")
      .body(Body::from(query))
      .unwrap();
    let response = graphql_request(req, state.as_ref()).await.unwrap();
    assert_eq!(response.status().as_u16(), 200);
  }

  async fn setup(&self) -> Arc<ServerContext> {
    let config = read_config_from_path(self.config.clone().as_str()).await.unwrap();
    let blueprint = Blueprint::try_from(&config).unwrap();
    let client = Arc::new(MockHttpClient { upstream_mocks: Arc::new(self.upstream_mocks.to_vec()) });
    let server_context = ServerContext::new(blueprint, client);
    Arc::new(server_context)
  }
}

#[derive(Clone)]
struct MockHttpClient {
  upstream_mocks: Arc<Vec<(UpstreamRequest, UpstreamResponse)>>,
}
#[async_trait::async_trait]
impl HttpClient for MockHttpClient {
  async fn execute(&self, req: reqwest::Request) -> anyhow::Result<Response> {
    let upstream_mocks = self.upstream_mocks.clone();
    let upstream_mock = upstream_mocks.iter().find(|(upstream_request, _res)| {
      let method_match = req.method().as_str() == upstream_request.0.method.as_str();
      let url_match = req.url().as_str() == req.uri().to_string().as_str();
      method_match && url_match
    });
    let upstream_response = upstream_mock.unwrap().clone().1.clone();
    let mut response =
      Response { status: reqwest::StatusCode::from_u16(upstream_response.0.status).unwrap(), ..Default::default() };
    let headers = upstream_response.0.headers.unwrap_or_default();
    for (k, v) in headers.iter() {
      response.headers.insert(
        HeaderName::from_str(k.as_str()).unwrap(),
        HeaderValue::from_str(v.as_str()).unwrap(),
      );
    }
    response.body = upstream_response.0.body.unwrap_or_default().to_value();
    Ok(response)
  }
}
#[cfg(test)]
mod test {
  use crate::APISpecification;

  #[tokio::test]
  async fn test_status_code() {
    let spec = APISpecification::read("tests/data/sample.json").unwrap();
    let query_string = "{\"operationName\":null,\"variables\":{},\"query\":\"{user {name}}\"}".to_string();

    spec.status(query_string).await;
  }
}
