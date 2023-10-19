extern crate core;

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Once};

use anyhow::anyhow;
use async_graphql_value::ConstValue;
use derive_setters::Setters;
use hyper::{Body, Request};
use reqwest::header::{HeaderName, HeaderValue};
use serde::Deserialize;
use serde_json::Value;
use tailcall::blueprint::Blueprint;
use tailcall::config::{Config, Source};
use tailcall::http::{graphql_single_request, HttpClient, Method, Response, ServerContext};
use url::Url;

static INIT: Once = Once::new();

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
}

#[derive(Default, Deserialize, Clone, Setters)]
pub struct HttpSpec {
  pub config: String,
  #[serde(skip)]
  path: PathBuf,
  pub name: String,
  #[serde(default)]
  pub description: Option<String>,
  pub upstream_mocks: Vec<(UpstreamRequest, UpstreamResponse)>,
  #[serde(default)]
  pub expected_upstream_requests: Vec<UpstreamRequest>,
  pub downstream_assertions: Vec<DownstreamAssertion>,
  pub annotation: Option<Annotation>,
}

impl HttpSpec {
  fn cargo_read(path: &str) -> anyhow::Result<Vec<HttpSpec>> {
    let dir_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path);
    let mut files = Vec::new();

    for entry in fs::read_dir(&dir_path)? {
      let path = entry?.path();
      if path.is_dir() {
        continue;
      }
      let source = Source::detect(path.to_str().unwrap_or_default())?;
      if path.is_file() && (source.ext() == "json" || source.ext() == "yml") {
        let contents = fs::read_to_string(&path)?;
        let spec: HttpSpec = Self::from_source(source, contents)?;
        files.push(spec.path(path));
      }
    }

    assert!(
      !files.is_empty(),
      "No files found in {}",
      dir_path.to_str().unwrap_or_default()
    );
    Ok(files)
  }

  fn filter_specs(specs: Vec<HttpSpec>) -> Vec<HttpSpec> {
    let mut only_specs = Vec::new();
    let mut filtered_specs = Vec::new();

    for spec in specs {
      match spec.annotation {
        Some(Annotation::Skip) => log::warn!("{} {} ... skipped", spec.name, spec.path.display()),
        Some(Annotation::Only) => only_specs.push(spec),
        Some(Annotation::Fail) => filtered_specs.push(spec),
        None => filtered_specs.push(spec),
      }
    }

    // If any spec has the Only annotation, use those; otherwise, use the filtered list.
    if !only_specs.is_empty() {
      only_specs
    } else {
      filtered_specs
    }
  }
  fn from_source(source: Source, contents: String) -> anyhow::Result<Self> {
    INIT.call_once(|| {
      env_logger::builder()
        .filter(Some("http_spec"), log::LevelFilter::Info)
        .init();
    });

    let spec = match source {
      Source::Json => anyhow::Ok(serde_json::from_str(&contents)?),
      Source::Yml => anyhow::Ok(serde_yaml::from_str(&contents)?),
      _ => Err(anyhow!("only json and yaml are supported")),
    };
    anyhow::Ok(spec?)
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

async fn assert_downstream(spec: HttpSpec) {
  for downstream_assertion in spec.downstream_assertions.iter() {
    if let Some(Annotation::Fail) = spec.annotation {
      let response = run(spec.clone(), &downstream_assertion).await.unwrap();
      let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
      assert_ne!(
        body,
        serde_json::to_string(&downstream_assertion.response.0.body).unwrap()
      )
    } else {
      let response = run(spec.clone(), &downstream_assertion).await.unwrap();
      let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
      assert_eq!(
        body,
        serde_json::to_string(&downstream_assertion.response.0.body).unwrap()
      )
    }
  }
  log::info!("{} {} ... ok", spec.name, spec.path.display());
}
#[tokio::test]
async fn test_body() -> std::io::Result<()> {
  let spec = HttpSpec::cargo_read("tests/http").unwrap();
  let spec = HttpSpec::filter_specs(spec);
  let tasks: Vec<_> = spec
    .into_iter()
    .map(|spec| tokio::spawn(async move { assert_downstream(spec).await }))
    .collect();
  for task in tasks {
    task.await?;
  }
  Ok(())
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
  graphql_single_request(req?, state.as_ref()).await
}
