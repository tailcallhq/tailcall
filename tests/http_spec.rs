use std::collections::BTreeMap;
use std::fs;
use std::str::FromStr;
use std::sync::Arc;

use async_graphql_value::ConstValue;
use hyper::{Body, Request};
use reqwest::header::{HeaderName, HeaderValue};
use serde::Deserialize;
use serde_json::Value;
use tailcall::blueprint::Blueprint;
use tailcall::config::Config;
use tailcall::http::{graphql_request, HttpClient, Method, Response, ServerContext};
use url::Url;

#[derive(Deserialize, Clone, Debug)]
pub enum Annotation {
  Skip,
  Only,
  Fail,
}
#[derive(Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct APIRequest {
  #[serde(default)]
  method: Method,
  pub url: Url,
  #[serde(default)]
  pub headers: BTreeMap<String, String>,
  #[serde(default)]
  pub body: serde_json::Value,
}
#[derive(Deserialize, Clone, Debug)]
pub struct APIResponse {
  pub status: u16,
  #[serde(default)]
  pub headers: BTreeMap<String, String>,
  #[serde(default)]
  pub body: serde_json::Value,
}
#[derive(Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct UpstreamRequest(pub APIRequest);
#[derive(Deserialize, Clone, Debug)]
pub struct UpstreamResponse(APIResponse);
#[derive(Deserialize, Clone)]
pub struct DownstreamRequest(pub APIRequest);
#[derive(Deserialize, Clone)]
pub struct DownstreamResponse(pub APIResponse);
#[derive(Deserialize, Clone)]
pub struct DownstreamAssertion {
  pub request: DownstreamRequest,
  pub response: DownstreamResponse,
  pub annotation: Option<Annotation>,
}

#[derive(Default, Deserialize, Clone)]
pub struct HttpSpec {
  pub config: String,
  pub name: String,
  pub description: Option<String>,
  pub upstream_mocks: Vec<(UpstreamRequest, UpstreamResponse)>,
  pub expected_upstream_requests: Vec<UpstreamRequest>,
  pub downstream_assertions: Vec<DownstreamAssertion>,
}

impl HttpSpec {
  fn read(spec: &str) -> Option<Self> {
    spec
      .split('.')
      .last()
      .and_then(|ext| match ext.to_lowercase().as_str() {
        "json" => Self::read_json(spec),
        "yaml" => Self::read_yaml(spec),
        _ => None,
      })
  }
  fn read_json(spec: &str) -> Option<Self> {
    let contents = fs::read_to_string(spec).ok()?;
    let spec = serde_json::from_str(&contents);

    spec.ok()
  }

  fn read_yaml(spec: &str) -> Option<Self> {
    let contents = fs::read_to_string(spec).ok()?;
    let spec = serde_yaml::from_str(&contents);
    spec.ok()
  }
  async fn setup(&self) -> Arc<ServerContext> {
    let config = Config::from_file_paths([self.config.clone()].iter())
      .await
      .ok()
      .unwrap();
    let blueprint = Blueprint::try_from(&config).unwrap();
    let client = Arc::new(MockHttpClient {
      upstream_mocks: self.upstream_mocks.to_vec(),
      expected_upstream_requests: self.expected_upstream_requests.to_vec(),
    });
    let server_context = ServerContext::new(blueprint, client);
    Arc::new(server_context)
  }
}

#[derive(Clone)]
struct MockHttpClient {
  upstream_mocks: Vec<(UpstreamRequest, UpstreamResponse)>,
  expected_upstream_requests: Vec<UpstreamRequest>,
}
#[async_trait::async_trait]
impl HttpClient for MockHttpClient {
  async fn execute(&self, req: reqwest::Request) -> Result<Response, anyhow::Error> {
    // Clone the mocks to allow iteration without borrowing issues.
    let mocks = self.upstream_mocks.clone();

    // Try to find a matching mock for the incoming request.
    let mock = mocks
      .iter()
      .find(|(mock_req, _)| {
        let method_match = req.method().as_str()
          == serde_json::to_string(&mock_req.0.method.clone())
            .expect("provided method is not valid")
            .as_str()
            .trim_matches('"');
        let url_match = req.url().as_str() == mock_req.0.url.clone().as_str();
        method_match && url_match
      })
      .expect("Mock not found");
    // Assert upstream request
    let upstream_request = mock.0.clone();
    assert!(
      self.expected_upstream_requests.contains(&upstream_request),
      "Unexpected upstream request: {:?}",
      upstream_request
    );

    // Clone the response from the mock to avoid borrowing issues.
    let mock_response = mock.1.clone();

    // Build the response with the status code from the mock.
    let status_code = reqwest::StatusCode::from_u16(mock_response.0.status)?;
    let mut response = Response { status: status_code, ..Default::default() };

    // Insert headers from the mock into the response.
    for (key, value) in mock_response.0.headers {
      let header_name = HeaderName::from_str(&key)?;
      let header_value = HeaderValue::from_str(&value)?;
      response.headers.insert(header_name, header_value);
    }

    // Set the body of the response.
    response.body = ConstValue::try_from(serde_json::from_value::<Value>(mock_response.0.body)?)?;

    Ok(response)
  }
}

#[tokio::test]
async fn test_body_yaml() {
  let spec = HttpSpec::read("tests/data/sample.yaml").unwrap();
  assert_downstream(spec).await;
}

async fn assert_downstream(spec: HttpSpec) {
  let has_only_annotation = spec
    .downstream_assertions
    .iter()
    .any(|assertion| matches!(assertion.annotation, Some(Annotation::Only)));

  for downstream_assertion in spec.downstream_assertions.iter() {
    match &downstream_assertion.annotation {
      Some(Annotation::Skip) if !has_only_annotation => {
        let request_details = format_request_details(&downstream_assertion.request.0);
        println!("Skipping test in: {}\nRequest Details: {}", spec.name, request_details);
        continue;
      }
      Some(Annotation::Only) | None
        if !has_only_annotation || matches!(downstream_assertion.annotation, Some(Annotation::Only)) =>
      {
        let response = run(spec.clone(), &downstream_assertion).await.unwrap();
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        assert_eq!(
          body,
          serde_json::to_string(&downstream_assertion.response.0.body).unwrap()
        );
      }
      Some(Annotation::Fail) if !has_only_annotation => {
        let response = run(spec.clone(), &downstream_assertion).await;
        assert!(response.is_err());
      }
      _ => {} // Skip other cases if "Only" is present in any assertion
    }
  }
}

// Helper function to format request details for printing.
fn format_request_details(request: &APIRequest) -> String {
  format!(
    "Method: {:?}, Path: {}, Body: {}",
    request.method.clone(),
    request.url.clone(),
    request.body.clone()
  )
}

#[tokio::test]
async fn test_body_json() {
  let spec = HttpSpec::read("tests/data/sample.json").unwrap();
  assert_downstream(spec).await;
}

async fn run(spec: HttpSpec, downstream_assertion: &&DownstreamAssertion) -> anyhow::Result<hyper::Response<Body>> {
  let query_string = serde_json::to_string(&downstream_assertion.request.0.body).expect("body is required");
  let method = downstream_assertion.request.0.method.clone();
  let url = downstream_assertion.request.0.url.clone();
  let state = spec.setup().await;
  let req = Request::builder()
    .method(method)
    .uri(url.as_str())
    .body(Body::from(query_string));
  graphql_request(req?, state.as_ref()).await
}
