extern crate core;

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Once};

use anyhow::{anyhow, Context};
use async_graphql_value::ConstValue;
use derive_setters::Setters;
use hyper::body::Bytes;
use hyper::{Body, Request};
use pretty_assertions::assert_eq;
use reqwest::header::{HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tailcall::blueprint::Blueprint;
use tailcall::config::introspection::{GraphqlConfigValidator, IntrospectionResult};
use tailcall::config::{Config, Source};
use tailcall::http::{graphql_batch_request, graphql_single_request, HttpClient, Method, Response, ServerContext};
use url::Url;

static INIT: Once = Once::new();

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
enum Annotation {
  Skip,
  Only,
  Fail,
}
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
struct APIRequest {
  #[serde(default)]
  method: Method,
  url: Url,
  #[serde(default)]
  headers: BTreeMap<String, String>,
  #[serde(default)]
  body: serde_json::Value,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct APIResponse {
  #[serde(default = "default_status")]
  status: u16,
  #[serde(default)]
  headers: BTreeMap<String, String>,
  #[serde(default)]
  body: serde_json::Value,
}
fn default_status() -> u16 {
  200
}
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
struct UpstreamRequest(APIRequest);
#[derive(Serialize, Deserialize, Clone, Debug)]
struct UpstreamResponse(APIResponse);
#[derive(Serialize, Deserialize, Clone)]
struct DownstreamRequest(APIRequest);
#[derive(Serialize, Deserialize, Clone)]
struct DownstreamResponse(APIResponse);
#[derive(Serialize, Deserialize, Clone)]
struct DownstreamAssertion {
  request: DownstreamRequest,
  response: DownstreamResponse,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
enum ConfigSource {
  File(String),
  Inline(Config),
}

#[derive(Serialize, Deserialize, Clone)]
struct Mock {
  request: UpstreamRequest,
  response: UpstreamResponse,
}

#[derive(Serialize, Deserialize, Clone, Setters)]
#[serde(rename_all = "camelCase")]
struct HttpSpec {
  config: ConfigSource,
  #[serde(skip)]
  path: PathBuf,
  name: String,
  description: Option<String>,

  #[serde(default)]
  mock: Vec<Mock>,

  #[serde(default)]
  expected_upstream_requests: Vec<UpstreamRequest>,
  assert: Vec<DownstreamAssertion>,

  // Annotations for the runner
  runner: Option<Annotation>,
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
        let spec: HttpSpec =
          Self::from_source(source, contents).map_err(|err| err.context(path.to_str().unwrap().to_string()))?;
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
      match spec.runner {
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

  async fn server_context(&self) -> Arc<ServerContext> {
    let config = match self.config.clone() {
      ConfigSource::File(file) => {
        Config::from_file_paths_with_validator([file].iter(), HttpSpec::mock_graphql_config_validator())
          .await
          .ok()
          .unwrap()
      }
      ConfigSource::Inline(config) => config,
    };
    let blueprint = Blueprint::try_from(&config).unwrap();
    let client = Arc::new(MockHttpClient { mocks: self.mock.to_vec() });
    let server_context = ServerContext::new(blueprint, client);
    Arc::new(server_context)
  }

  fn mock_graphql_config_validator() -> GraphqlConfigValidator {
    let contents = fs::read_to_string("tests/data/introspection-result.json").unwrap();
    let introspection_result: IntrospectionResult = serde_json::from_str(contents.as_str()).unwrap();
    let mut cache = BTreeMap::new();
    cache.insert("http://upstream/graphql".to_string(), introspection_result);

    GraphqlConfigValidator::with_values(cache)
  }
}

#[derive(Clone)]
struct MockHttpClient {
  mocks: Vec<Mock>,
}
#[async_trait::async_trait]
impl HttpClient for MockHttpClient {
  async fn execute(&self, req: reqwest::Request) -> Result<Response, anyhow::Error> {
    // Clone the mocks to allow iteration without borrowing issues.
    let mocks = self.mocks.clone();

    // Try to find a matching mock for the incoming request.
    let mock = mocks
      .iter()
      .find(|Mock { request: mock_req, response: _ }| {
        let method_match = req.method().as_str()
          == serde_json::to_string(&mock_req.0.method.clone())
            .expect("provided method is not valid")
            .as_str()
            .trim_matches('"');
        let url_match = req.url().as_str() == mock_req.0.url.clone().as_str();
        let req_body = match req.body() {
          Some(body) => {
            if let Some(bytes) = body.as_bytes() {
              if let Ok(body_str) = std::str::from_utf8(bytes) {
                Value::from(body_str)
              } else {
                Value::Null
              }
            } else {
              Value::Null
            }
          }
          None => Value::Null,
        };
        let body_match = req_body == mock_req.0.body;
        method_match && url_match && body_match
      })
      .unwrap_or_else(|| panic!("Unexpected upstream request: {:?}", req));

    // Clone the response from the mock to avoid borrowing issues.
    let mock_response = mock.response.clone();

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
  for assertion in spec.assert.iter() {
    if let Some(Annotation::Fail) = spec.runner {
      let response = run(spec.clone(), &assertion).await.unwrap();
      let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
      assert_eq!(body, serde_json::to_string(&assertion.response.0.body).unwrap());
      log::error!("{} {} ... failed", spec.name, spec.path.display());
      panic!(
        "Expected spec: {} {} to fail but it passed",
        spec.name,
        spec.path.display()
      );
    } else {
      let response = run(spec.clone(), &assertion)
        .await
        .context(spec.path.to_str().unwrap().to_string())
        .unwrap();
      let actual_status = response.status().clone().as_u16();
      let actual_headers = assertion.response.0.headers.clone();
      let actual_body = hyper::body::to_bytes(response.into_body()).await.unwrap();

      // Assert Status
      assert_eq!(actual_status, assertion.response.0.status);

      // Assert Body
      assert_eq!(
        to_json_pretty(actual_body).unwrap(),
        serde_json::to_string_pretty(&assertion.response.0.body).unwrap()
      );

      // Assert Headers
      for (key, value) in assertion.response.0.headers.iter() {
        assert_eq!(actual_headers.get(key), Some(value));
      }
    }
  }
  log::info!("{} {} ... ok", spec.name, spec.path.display());
}

fn to_json_pretty(bytes: Bytes) -> anyhow::Result<String> {
  let body_str = String::from_utf8(bytes.to_vec())?;
  let json: Value = serde_json::from_str(&body_str)?;
  Ok(serde_json::to_string_pretty(&json)?)
}

#[tokio::test]
async fn http_spec_e2e() -> std::io::Result<()> {
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
  let server_context = spec.server_context().await;
  let req = Request::builder()
    .method(method)
    .uri(url.as_str())
    .body(Body::from(query_string))?;

  // TODO: reuse logic from server.rs to select the correct handler
  if server_context.blueprint.server.enable_batch_requests {
    graphql_batch_request(req, server_context.as_ref()).await
  } else {
    graphql_single_request(req, server_context.as_ref()).await
  }
}
