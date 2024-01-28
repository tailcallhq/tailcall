extern crate core;

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{Arc, Once};
use std::{fs, panic};

use anyhow::{anyhow, Context};
use derive_setters::Setters;
use futures_util::future::join_all;
use hyper::body::Bytes;
use hyper::{Body, Request};
use markdown::mdast::Node;
use markdown::ParseOptions;
use reqwest::header::{HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tailcall::async_graphql_hyper::{GraphQLBatchRequest, GraphQLRequest};
use tailcall::blueprint::Blueprint;
use tailcall::cli::{init_chrono_cache, init_file, init_hook_http, init_http};
use tailcall::config::reader::ConfigReader;
use tailcall::config::{Config, Source, Upstream};
use tailcall::http::{handle_request, AppContext, Method, Response};
use tailcall::{EnvIO, HttpIO};
use url::Url;

static INIT: Once = Once::new();

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
enum Annotation {
    Skip,
    Only,
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

pub struct Env {
    env: HashMap<String, String>,
}

impl EnvIO for Env {
    fn get(&self, key: &str) -> Option<String> {
        self.env.get(key).cloned()
    }
}

impl Env {
    pub fn init(map: HashMap<String, String>) -> Self {
        Self { env: map }
    }
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
struct DownstreamAssertion {
    request: DownstreamRequest,
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
struct AssertSpec {
    #[serde(default)]
    mock: Vec<Mock>,

    assert: Vec<DownstreamAssertion>,

    #[serde(default)]
    env: HashMap<String, String>,
}

#[derive(Clone, Setters, Debug)]
struct ExecutionSpec {
    path: PathBuf,
    name: String,
    safe_name: String,

    server: Vec<Config>,
    assert: Option<AssertSpec>,

    // Annotations for the runner
    runner: Option<Annotation>,
}

impl ExecutionSpec {
    async fn cargo_read(path: &str) -> anyhow::Result<Vec<ExecutionSpec>> {
        let dir_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path);
        let mut files = Vec::new();

        for entry in fs::read_dir(&dir_path)? {
            let path = entry?.path();
            if path.is_dir() {
                continue;
            }
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|x| x.to_str()) {
                    if ext == "md" {
                        let contents = fs::read_to_string(&path)?;
                        let spec: ExecutionSpec = Self::from_source(&path, contents)
                            .await
                            .map_err(|err| err.context(path.to_str().unwrap().to_string()))?;

                        files.push(spec.path(path));
                    }
                }
            }
        }

        assert!(
            !files.is_empty(),
            "No files found in {}",
            dir_path.to_str().unwrap_or_default()
        );
        Ok(files)
    }

    fn filter_specs(specs: Vec<ExecutionSpec>) -> Vec<ExecutionSpec> {
        let mut only_specs = Vec::new();
        let mut filtered_specs = Vec::new();

        for spec in specs {
            match spec.runner {
                Some(Annotation::Skip) => {
                    log::warn!("{} {} ... skipped", spec.name, spec.path.display())
                }
                Some(Annotation::Only) => only_specs.push(spec),
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

    async fn from_source(path: &Path, contents: String) -> anyhow::Result<Self> {
        INIT.call_once(|| {
            env_logger::builder()
                .filter(Some("execution_spec"), log::LevelFilter::Info)
                .init();
        });

        let ast = markdown::to_mdast(&contents, &ParseOptions::default()).unwrap();
        let mut children = ast
            .children()
            .unwrap_or_else(|| panic!("Failed to parse {:?}: empty file unexpected", path))
            .iter()
            .peekable();

        let mut name: Option<String> = None;
        let mut server: Vec<(Source, String)> = Vec::with_capacity(2);
        let mut assert: Option<AssertSpec> = None;
        let mut runner: Option<Annotation> = None;

        while let Some(node) = children.next() {
            match node {
                Node::Heading(heading) => {
                    if heading.depth == 1 {
                        // Parse test name
                        if name.is_none() {
                            if let Some(Node::Text(text)) = heading.children.first() {
                                name = Some(text.value.clone());
                            } else {
                                return Err(anyhow!(
                                    "Unexpected content of level 1 heading in {:?}: {:#?}",
                                    path,
                                    heading
                                ));
                            }
                        } else {
                            return Err(anyhow!(
                                "Unexpected double-declaration of test name in {:?}",
                                path
                            ));
                        }

                        // Consume optional test description
                        if let Some(Node::Paragraph(_)) = children.peek() {
                            let _ = children.next();
                        }
                    } else if heading.depth == 5 {
                        // Parse annotation
                        if runner.is_none() {
                            if let Some(Node::Text(text)) = heading.children.first() {
                                runner = Some(match text.value.as_str() {
                                    "skip" => Annotation::Skip,
                                    "only" => Annotation::Only,
                                    _ => {
                                        return Err(anyhow!(
                                            "Unexpected runner annotation {:?} in {:?}",
                                            text.value,
                                            path,
                                        ))
                                    }
                                });
                            } else {
                                return Err(anyhow!(
                                    "Unexpected content of level 3 heading in {:?}: {:#?}",
                                    path,
                                    heading
                                ));
                            }
                        } else {
                            return Err(anyhow!(
                                "Unexpected double-declaration of runner annotation in {:?}",
                                path
                            ));
                        }
                    } else if heading.depth == 4 {
                        // Parse following code hblock
                        let (content, lang) = if let Some(Node::Code(code)) = children.next() {
                            (code.value.to_owned(), code.lang.to_owned())
                        } else {
                            return Err(anyhow!("Unexpected non-code block node or EOF after component definition in {:?}", path));
                        };

                        let lang = match lang {
                            Some(x) => Ok(x),
                            None => {
                                Err(anyhow!("Unexpected languageless code block in {:?}", path))
                            }
                        }?;

                        let source = Source::from_str(&lang)?;

                        // Parse component name
                        if let Some(Node::Text(text)) = heading.children.first() {
                            let name = text.value.as_str();

                            match name {
                                "server:" => {
                                    // Server configs are only parsed if the test isn't skipped.
                                    server.push((source, content));
                                }
                                "assert:" => {
                                    assert = match source {
                                        Source::Json => anyhow::Ok(serde_json::from_str(&content)?),
                                        Source::Yml => anyhow::Ok(serde_yaml::from_str(&content)?),
                                        _ => Err(anyhow!("Unexpected language in assert block in {:?} (only JSON and YAML are supported)", path)),
                                    }?;
                                }
                                _ => {
                                    return Err(anyhow!(
                                        "Unexpected component {:?} in {:?}: {:#?}",
                                        name,
                                        path,
                                        heading
                                    ))
                                }
                            }
                        } else {
                            return Err(anyhow!(
                                "Unexpected content of level 4 heading in {:?}: {:#?}",
                                path,
                                heading
                            ));
                        }
                    } else {
                        return Err(anyhow!(
                            "Unexpected level {} heading in {:?}: {:#?}",
                            heading.depth,
                            path,
                            heading
                        ));
                    }
                }
                _ => return Err(anyhow!("Unexpected node in {:?}: {:#?}", path, node)),
            }
        }

        if server.is_empty() {
            return Err(anyhow!(
                "You must define a GraphQL Config in an execution test."
            ));
        }

        // Only parse configs if test isn't skipped.
        let server = if !matches!(runner, Some(Annotation::Skip)) {
            let mut s: Vec<Config> = Vec::with_capacity(server.len());

            for (source, content) in server.into_iter() {
                let config = Config::default();

                let new_config = Config::from_source(source, &content)
                    .expect(&format!("Couldn't parse GraphQL in {:#?}", path));

                let reader = ConfigReader::init(
                    init_file(),
                    init_http(&Upstream::default(), None),
                );
                
                let new_config = match reader.read_script(new_config).await {
                    Ok(x) => x,
                    Err(e) => return Err(anyhow!("Couldn't read scripts of GraphQL in {:#?}: {}", path, e))
                };

                s.push(config.merge_right(&new_config));
            }

            s
        } else {
            Vec::with_capacity(0) // Test is gonna be skipped anyways, so this doesn't matter.
        };

        let spec = ExecutionSpec {
            path: path.to_owned(),
            name: name.unwrap_or_else(|| path.file_name().unwrap().to_str().unwrap().to_string()),
            safe_name: path.file_name().unwrap().to_str().unwrap().to_string(),
            server,
            assert,
            runner,
        };

        anyhow::Ok(spec)
    }

    async fn server_context(
        &self,
        config: &Config,
        env: HashMap<String, String>,
    ) -> Arc<AppContext> {
        let blueprint = Blueprint::try_from(config).unwrap();
        let client = init_hook_http(
            MockHttpClient::new(self.clone()),
            blueprint.server.script.clone(),
        );
        let http2_client = Arc::new(MockHttpClient::new(self.clone()));
        let env = Arc::new(Env::init(env));
        let chrono_cache = Arc::new(init_chrono_cache());
        let server_context = AppContext::new(blueprint, client, http2_client, env, chrono_cache);
        Arc::new(server_context)
    }
}

#[derive(Clone)]
struct MockHttpClient {
    spec: ExecutionSpec,
}
impl MockHttpClient {
    fn new(spec: ExecutionSpec) -> Self {
        MockHttpClient { spec }
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
        let mocks = self.spec.assert.as_ref().unwrap().mock.clone();

        // Determine if the request is a GRPC request based on PORT
        let is_grpc = req.url().as_str().contains("50051");

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
                method_match && url_match && (body_match || is_grpc)
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

        // Special Handling for GRPC
        if is_grpc {
            let body = string_to_bytes(mock_response.0.body.as_str().unwrap());
            response.body = Bytes::from_iter(body);
            Ok(response)
        } else {
            let body = serde_json::to_vec(&mock_response.0.body)?;
            response.body = Bytes::from_iter(body);
            Ok(response)
        }
    }
}

async fn assert_spec(spec: ExecutionSpec) {
    log::info!("{} {} ...", spec.name, spec.path.display());

    if let Some(assert_spec) = spec.assert.as_ref() {
        for (i, assertion) in assert_spec.assert.iter().enumerate() {
            let response = run_assert(
                spec.clone(),
                assert_spec.clone(),
                &assertion,
                spec.server.first().unwrap(),
            )
            .await
            .context(spec.path.to_str().unwrap().to_string())
            .unwrap();

            let mut headers: BTreeMap<String, String> = BTreeMap::new();

            for (key, value) in response.headers() {
                headers.insert(key.to_string(), value.to_str().unwrap().to_string());
            }

            let response: APIResponse = APIResponse {
                status: response.status().clone().as_u16(),
                headers,
                body: serde_json::from_slice(
                    &hyper::body::to_bytes(response.into_body()).await.unwrap(),
                )
                .unwrap(),
            };

            let snapshot_name = format!("{}_assert_{}", spec.safe_name, i);

            insta::assert_json_snapshot!(snapshot_name, response);
        }
    }

    log::info!("{} {} ... ok", spec.name, spec.path.display());
}

#[tokio::test]
async fn execution_spec() -> anyhow::Result<()> {
    let spec = ExecutionSpec::cargo_read("tests/execution").await?;
    let spec = ExecutionSpec::filter_specs(spec);
    let tasks: Vec<_> = spec.into_iter().map(assert_spec).collect();
    join_all(tasks).await;
    Ok(())
}

async fn run_assert(
    spec: ExecutionSpec,
    assert: AssertSpec,
    downstream_assertion: &&DownstreamAssertion,
    config: &Config,
) -> anyhow::Result<hyper::Response<Body>> {
    let query_string =
        serde_json::to_string(&downstream_assertion.request.0.body).expect("body is required");
    let method = downstream_assertion.request.0.method.clone();
    let headers = downstream_assertion.request.0.headers.clone();
    let url = downstream_assertion.request.0.url.clone();
    let server_context = spec.server_context(config, assert.env.clone()).await;
    let req = headers
        .into_iter()
        .fold(
            Request::builder()
                .method(method.to_hyper())
                .uri(url.as_str()),
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
