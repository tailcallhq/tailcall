use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;

use derive_setters::Setters;
use hyper::{Body, Request, StatusCode, Uri};
use reqwest::header::HeaderMap;
use serde::Deserialize;
use serde_json::Value;
use tailcall::blueprint::Blueprint;
use tailcall::config::Config;
use tailcall::http::{graphql_request, Method, ServerContext};
use url::Url;
#[derive(Deserialize)]
struct APIRequest {
  method: Method,
  url: Url,
  headers: Option<BTreeMap<String, String>>,
  body: Option<serde_json::Value>,
}
#[derive(Deserialize)]
struct APIResponse {
  status: u16,
  headers: Option<BTreeMap<String, String>>,
  body: Option<serde_json::Value>,
}
#[derive(Deserialize)]
struct UpstreamRequest(APIRequest);
#[derive(Deserialize)]
struct UpstreamResponse(APIResponse);
#[derive(Deserialize)]
struct DownstreamRequest(APIRequest);
#[derive(Deserialize)]
struct DownstreamResponse(APIResponse);
#[derive(Deserialize)]
struct DownstreamAssertion {
  request: DownstreamRequest,
  response: DownstreamResponse,
}

#[derive(Default, Deserialize)]
struct APISpecification {
  config: String,
  name: String,
  description: Option<String>,
  upstream_mocks: Vec<(UpstreamRequest, UpstreamResponse)>,
  expected_upstream_requests: Vec<UpstreamRequest>,
  downstream_assertions: Vec<DownstreamAssertion>,
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
  async fn run(&self, query: String) -> () {
    println!("config{:?}", self.config);
    let config = read_config_from_path(self.config.clone().as_str()).await.unwrap();
    let blueprint = Blueprint::try_from(&config).unwrap();
    let server_context = ServerContext::new(blueprint);
    let state = Arc::new(server_context);
    let req = Request::builder()
      .method(Method::POST)
      .uri("http://localhost:8080/graphql")
      .body(Body::from(query))
      .unwrap();
    let response = graphql_request(req, state.as_ref())
      .await
      .map_err(|e| println!("{}", e))
      .unwrap();
    println!("{:?}", response);
    for assertion in self.downstream_assertions.iter() {
      println!("here");
      assert_eq!(response.status().as_u16(), assertion.response.0.status);
    }
  }
}

#[cfg(test)]
mod test {
  use crate::APISpecification;

  #[tokio::test]
  async fn upstream_request_headers() {
    let spec = APISpecification::read("tests/data/sample.json").unwrap();
    let query_string = "{\"operationName\":null,\"variables\":{},\"query\":\"{user {name}}\"}".to_string();

    spec.run(query_string).await;
  }
}
