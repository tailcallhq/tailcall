extern crate core;

use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};
use std::panic;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use anyhow::{anyhow, Context};
use hyper::body::Bytes;
use reqwest::header::{HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tailcall::cache::InMemoryCache;
use tailcall::cli::javascript;
use tailcall::http::{Method, Response};
use tailcall::runtime::TargetRuntime;
use tailcall::{blueprint, EnvIO, FileIO, HttpIO};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use url::Url;

use super::model::ExecutionSpec;

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct APIRequest {
    #[serde(default)]
    pub method: Method,
    pub url: Url,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    #[serde(default)]
    pub body: serde_json::Value,
    #[serde(default)]
    pub test_traces: bool,
    #[serde(default)]
    pub test_metrics: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct APIResponse {
    #[serde(default = "default_status")]
    pub status: u16,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    #[serde(default)]
    pub body: serde_json::Value,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_body: Option<String>,
}

fn default_status() -> u16 {
    200
}

fn default_expected_hits() -> usize {
    1
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
struct UpstreamRequest(APIRequest);

#[derive(Serialize, Deserialize, Clone, Debug)]
struct UpstreamResponse(APIResponse);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Mock {
    request: UpstreamRequest,
    response: UpstreamResponse,
    #[serde(default = "default_expected_hits")]
    expected_hits: usize,
}

#[derive(Clone, Debug)]
struct ExecutionMock {
    mock: Mock,
    actual_hits: Arc<AtomicUsize>,
}

impl ExecutionMock {
    fn test_hits(&self, path: impl AsRef<Path>) {
        let url = &self.mock.request.0.url;
        let is_batch_graphql = url.path().starts_with("/graphql")
            && self
                .mock
                .request
                .0
                .body
                .as_str()
                .map(|s| s.contains(','))
                .unwrap_or_default();

        // do not test hits for mocks for batch graphql requests
        // since that requires having 2 mocks with different order of queries in
        // single request and only one of that mocks is actually called during run.
        // for other protocols there is no issues right now, because:
        // - for http the keys are always sorted https://github.com/tailcallhq/tailcall/blob/51d8b7aff838f0f4c362d4ee9e39492ae1f51fdb/src/http/data_loader.rs#L71
        // - for grpc body is not used for matching the mock and grpc will use grouping based on id https://github.com/tailcallhq/tailcall/blob/733b641c41f17c60b15b36b025b4db99d0f9cdcd/tests/execution_spec.rs#L769
        if is_batch_graphql {
            return;
        }

        let expected_hits = self.mock.expected_hits;
        let actual_hits = self.actual_hits.load(Ordering::Relaxed);

        assert_eq!(
            expected_hits,
            actual_hits,
            "expected mock for {url} to be hit exactly {expected_hits} times, but it was hit {actual_hits} times for file: {:?}",
            path.as_ref()
        );
    }
}

#[derive(Clone, Debug)]
pub struct MockHttpClient {
    mocks: Vec<ExecutionMock>,
    spec_path: String,
}

impl MockHttpClient {
    pub fn new(spec: &ExecutionSpec) -> Self {
        let mocks = spec
            .mock
            .as_ref()
            .map(|mocks| {
                mocks
                    .iter()
                    .map(|mock| ExecutionMock {
                        mock: mock.clone(),
                        actual_hits: Arc::new(AtomicUsize::default()),
                    })
                    .collect()
            })
            .unwrap_or_default();

        let spec_path = spec
            .path
            .strip_prefix(std::env::current_dir().unwrap())
            .unwrap_or(&spec.path)
            .to_string_lossy()
            .into_owned();

        MockHttpClient { mocks, spec_path }
    }

    pub fn test_hits(&self, path: impl AsRef<Path>) {
        for mock in &self.mocks {
            mock.test_hits(path.as_ref());
        }
    }
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
impl HttpIO for MockHttpClient {
    async fn execute(&self, req: reqwest::Request) -> anyhow::Result<Response<Bytes>> {
        // Determine if the request is a GRPC request based on PORT
        let is_grpc = req.url().as_str().contains("50051");

        // Try to find a matching mock for the incoming request.
        let execution_mock = self
            .mocks
            .iter()
            .find(|mock| {
                let mock_req = &mock.mock.request;
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
                let headers_match = req
                    .headers()
                    .iter()
                    .filter(|(key, _)| *key != "content-type")
                    .all(|(key, value)| {
                        let header_name = key.to_string();

                        let header_value = value.to_str().unwrap();
                        let mock_header_value = "".to_string();
                        let mock_header_value = mock_req
                            .0
                            .headers
                            .get(&header_name)
                            .unwrap_or(&mock_header_value);
                        header_value == mock_header_value
                    });
                method_match && url_match && headers_match && (body_match || is_grpc)
            })
            .ok_or(anyhow!(
                "No mock found for request: {:?} {} in {}",
                req.method(),
                req.url(),
                self.spec_path
            ))?;

        execution_mock.actual_hits.fetch_add(1, Ordering::Relaxed);

        // Clone the response from the mock to avoid borrowing issues.
        let mock_response = execution_mock.mock.response.clone();

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

        // Special Handling for GRPC
        if let Some(body) = mock_response.0.text_body {
            // Return plaintext body if specified
            let body = string_to_bytes(&body);
            response.body = Bytes::from_iter(body);
        } else if is_grpc {
            // Special Handling for GRPC
            let body = string_to_bytes(mock_response.0.body.as_str().unwrap_or_default());
            response.body = Bytes::from_iter(body);
        } else {
            let body = serde_json::to_vec(&mock_response.0.body)?;
            response.body = Bytes::from_iter(body);
        }

        Ok(response)
    }
}

pub struct MockFileSystem {
    spec: ExecutionSpec,
}

impl MockFileSystem {
    pub fn new(spec: ExecutionSpec) -> MockFileSystem {
        MockFileSystem { spec }
    }
}

#[async_trait::async_trait]
impl FileIO for MockFileSystem {
    async fn write<'a>(&'a self, _path: &'a str, _content: &'a [u8]) -> anyhow::Result<()> {
        Err(anyhow!("Cannot write to a file in an execution spec"))
    }

    async fn read<'a>(&'a self, path: &'a str) -> anyhow::Result<String> {
        let base = PathBuf::from(path);
        let path = base
            .file_name()
            .context("Invalid file path")?
            .to_str()
            .context("Invalid OsString")?;
        match self.spec.files.get(path) {
            Some(x) => Ok(x.to_owned()),
            None => Err(anyhow!("No such file or directory (os error 2)")),
        }
    }
}

#[derive(Clone)]
struct TestFileIO {}

impl TestFileIO {
    fn init() -> Self {
        TestFileIO {}
    }
}

#[async_trait::async_trait]
impl FileIO for TestFileIO {
    async fn write<'a>(&'a self, path: &'a str, content: &'a [u8]) -> anyhow::Result<()> {
        let mut file = tokio::fs::File::create(path).await?;
        file.write_all(content)
            .await
            .map_err(|e| anyhow!("{}", e))?;
        Ok(())
    }

    async fn read<'a>(&'a self, path: &'a str) -> anyhow::Result<String> {
        let mut file = tokio::fs::File::open(path).await?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .await
            .map_err(|e| anyhow!("{}", e))?;
        Ok(String::from_utf8(buffer)?)
    }
}

#[derive(Clone)]
struct TestEnvIO {
    vars: HashMap<String, String>,
}

impl EnvIO for TestEnvIO {
    fn get(&self, key: &str) -> Option<Cow<'_, str>> {
        self.vars.get(key).map(Cow::from)
    }
}

impl TestEnvIO {
    pub fn init(vars: Option<HashMap<String, String>>) -> Self {
        Self { vars: vars.unwrap_or_default() }
    }
}

pub fn create_runtime(
    http_client: Arc<MockHttpClient>,
    env: Option<HashMap<String, String>>,
    script: Option<blueprint::Script>,
) -> TargetRuntime {
    let http = if let Some(script) = script.clone() {
        javascript::init_http(http_client.clone(), script)
    } else {
        http_client.clone()
    };

    let http2 = if let Some(script) = script {
        javascript::init_http(http_client.clone(), script)
    } else {
        http_client.clone()
    };

    let file = TestFileIO::init();
    let env = TestEnvIO::init(env);

    TargetRuntime {
        http,
        http2_only: http2,
        env: Arc::new(env),
        file: Arc::new(file),
        cache: Arc::new(InMemoryCache::new()),
        extensions: Arc::new(vec![]),
    }
}
