extern crate core;

use std::collections::{BTreeMap, HashMap};
use std::panic::AssertUnwindSafe;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Once};
use std::{fs, panic};

use anyhow::{anyhow, Context};
use async_graphql_value::ConstValue;
use derive_setters::Setters;
use hyper::body::Bytes;
use hyper::{Body, Request};
use pretty_assertions::assert_eq;
use reqwest::header::{HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tc_core::async_graphql_hyper::{GraphQLBatchRequest, GraphQLRequest};
use tc_core::blueprint::Blueprint;
use tc_core::config::{Config, Source};
use tc_core::http::{handle_request, HttpClient, Method, Response, server_context::ServerContext};
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
#[derive(Serialize, Deserialize, Clone, Debug)]
struct DownstreamRequest(APIRequest);
#[derive(Serialize, Deserialize, Clone, Debug)]
struct DownstreamResponse(APIResponse);
#[derive(Serialize, Deserialize, Clone, Debug)]
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

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Mock {
  request: UpstreamRequest,
  response: UpstreamResponse,
}

#[derive(Serialize, Deserialize, Clone, Setters, Debug)]
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
  env: HashMap<String, String>,

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

    let spec: HttpSpec = match source {
      Source::Json => anyhow::Ok(serde_json::from_str(&contents)?),
      Source::Yml => anyhow::Ok(serde_yaml::from_str(&contents)?),
      _ => Err(anyhow!("only json and yaml are supported")),
    }?;

    anyhow::Ok(spec)
  }

  async fn server_context(&self) -> Arc<ServerContext> {
    let config = match self.config.clone() {
      ConfigSource::File(file) => Config::read_from_files([file].iter()).await.unwrap(),
      ConfigSource::Inline(config) => config,
    };
    let blueprint = Blueprint::try_from(&config).unwrap();
    let client = Arc::new(MockHttpClient { spec: self.clone() });
    let http2_client = Arc::new(MockHttpClient { spec: self.clone() });
    let mut server_context = ServerContext::with_http_clients(blueprint, client, http2_client);
    server_context.env_vars = Arc::new(self.env.clone());

    Arc::new(server_context)
  }
}

#[derive(Clone)]
struct MockHttpClient {
  spec: HttpSpec,
}

fn string_to_bytes(input: &str) -> Vec<u8> {
  let mut bytes = Vec::new();
  let mut chars = input.chars().peekable();

  while let Some(c) = chars.next() {
    match c {
      '\\' => match chars.next() {
        Some('0') => bytes.push(0),
        Some('n') => bytes.push(b'\n'),
        Some('t') => bytes.push(b'\t'),
        Some('r') => bytes.push(b'\r'),
        Some('\\') => bytes.push(b'\\'),
        Some('\"') => bytes.push(b'\"'),
        Some('x') => {
          let mut hex = chars.next().unwrap().to_string();
          hex.push(chars.next().unwrap());
          let byte = u8::from_str_radix(&hex, 16).unwrap();
          bytes.push(byte);
        }
        _ => panic!("Unsupported escape sequence"),
      },
      _ => bytes.push(c as u8),
    }
  }

  bytes
}
#[async_trait::async_trait]
impl HttpClient for MockHttpClient {
  async fn execute(&self, req: reqwest::Request) -> anyhow::Result<Response> {
    // Clone the mocks to allow iteration without borrowing issues.
    let mocks = self.spec.mock.clone();

    // Try to find a matching mock for the incoming request.
    let mock = mocks
      .iter()
      .find(|Mock { request: mock_req, response: _ }| {
        let method_match = req.method() == mock_req.0.method.clone().to_hyper();
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
      .ok_or(anyhow!(
        "No mock found for request: {:?} {} in {}",
        req.method(),
        req.url(),
        format!("{}", self.spec.path.to_str().unwrap())
      ))?;

    // Clone the response from the mock to avoid borrowing issues.
    let mock_response = mock.response.clone();

    // Build the response with the status code from the mock.
    let status_code = reqwest::StatusCode::from_u16(mock_response.0.status)?;

    if status_code.is_client_error() || status_code.is_server_error() {
      return Err(anyhow::format_err!("Status code error"));
    }

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

  async fn execute_raw(&self, req: reqwest::Request) -> anyhow::Result<reqwest::Response> {
    let mocks = self.spec.mock.clone();

    // Try to find a matching mock for the incoming request.
    let mock = mocks
      .iter()
      .find(|Mock { request: mock_req, response: _ }| {
        let method_match = req.method() == mock_req.0.method.clone().to_hyper();
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
        let _body_match = req_body == mock_req.0.body;
        method_match && url_match // && body_match
      })
      .ok_or(anyhow!(
        "No mock found for request: {:?} {} in {}",
        req.method(),
        req.url(),
        format!("{}", self.spec.path.to_str().unwrap())
      ))?;

    // Clone the response from the mock to avoid borrowing issues.
    let mock_response = mock.response.clone();

    // Build the response with the status code from the mock.
    let status_code = reqwest::StatusCode::from_u16(mock_response.0.status)?;

    if status_code.is_client_error() || status_code.is_server_error() {
      return Err(anyhow::format_err!("Status code error"));
    }

    let mut response = hyper::Response::builder().status(status_code);
    let headers = response.headers_mut().ok_or(anyhow!("Invalid headers"))?;
    // Insert headers from the mock into the response.
    for (key, value) in mock_response.0.headers {
      let header_name = HeaderName::from_str(&key)?;
      let header_value = HeaderValue::from_str(&value)?;
      headers.insert(header_name, header_value);
    }

    let body = mock_response.0.body.as_str().unwrap_or_default();
    let res = response.body(Body::from(string_to_bytes(body)))?;

    Ok(reqwest::Response::from(res))
  }
}

async fn assert_downstream(spec: HttpSpec) {
  for assertion in spec.assert.iter() {
    if let Some(Annotation::Fail) = spec.runner {
      let response = run(spec.clone(), &assertion).await.unwrap();
      let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
      let result = panic::catch_unwind(AssertUnwindSafe(|| {
        assert_eq!(
          body,
          serde_json::to_string(&assertion.response.0.body).unwrap(),
          "File: {} {}",
          spec.name,
          spec.path.display()
        );
      }));

      match result {
        Ok(_) => {
          panic!(
            "Expected spec: {} {} to fail but it passed",
            spec.name,
            spec.path.display()
          );
        }
        Err(_) => {
          log::info!("{} {} ... failed (expected)", spec.name, spec.path.display());
        }
      }
    } else {
      let response = run(spec.clone(), &assertion)
        .await
        .context(spec.path.to_str().unwrap().to_string())
        .unwrap();
      let actual_status = response.status().clone().as_u16();
      let actual_headers = response.headers().clone();
      let actual_body = hyper::body::to_bytes(response.into_body()).await.unwrap();

      // Assert Status
      assert_eq!(
        actual_status,
        assertion.response.0.status,
        "File: {} {}",
        spec.name,
        spec.path.display()
      );

      // Assert Body
      assert_eq!(
        to_json_pretty(actual_body).unwrap(),
        serde_json::to_string_pretty(&assertion.response.0.body).unwrap(),
        "File: {} {}",
        spec.name,
        spec.path.display()
      );

      // Assert Headers
      for (key, value) in assertion.response.0.headers.iter() {
        match actual_headers.get(key) {
          None => panic!("Expected header {} to be present", key),
          Some(actual_value) => assert_eq!(actual_value, value, "File: {} {}", spec.name, spec.path.display()),
        }
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
async fn http_spec_e2e() -> anyhow::Result<()> {
  let spec = HttpSpec::cargo_read("tests/http")?;
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
  let headers = downstream_assertion.request.0.headers.clone();
  let url = downstream_assertion.request.0.url.clone();
  let server_context = spec.server_context().await;
  let req = headers
    .into_iter()
    .fold(
      Request::builder().method(method.to_hyper()).uri(url.as_str()),
      |acc, (key, value)| acc.header(key, value),
    )
    .body(Body::from(query_string))?;

  // TODO: reuse logic from server.rs to select the correct handler
  if server_context.blueprint.server.enable_batch_requests {
    handle_request::<GraphQLBatchRequest>(req, server_context).await
  } else {
    handle_request::<GraphQLRequest>(req, server_context).await
  }
}
