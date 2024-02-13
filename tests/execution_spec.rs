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
use tailcall::builder::{TailcallBuilder, TailcallExecutor};
use tailcall::cache::InMemoryCache;
use tailcall::cli::javascript;
use tailcall::config::{Config, Source};
use tailcall::http::{AppContext, Method, Response};
use tailcall::print_schema::print_schema;
use tailcall::runtime::TargetRuntime;
use tailcall::valid::{Cause, ValidationError};
use tailcall::{EnvIO, FileIO, HttpIO};
use url::Url;

#[cfg(test)]
pub mod test {
    use std::collections::HashMap;
    use std::sync::Arc;

    use anyhow::{anyhow, Result};
    use hyper::body::Bytes;
    use reqwest::Client;
    use tailcall::cache::InMemoryCache;
    use tailcall::http::Response;
    use tailcall::runtime::TargetRuntime;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    use crate::{EnvIO, FileIO, HttpIO};

    #[derive(Clone)]
    struct TestHttp {
        client: Client,
    }

    impl Default for TestHttp {
        fn default() -> Self {
            Self { client: Client::new() }
        }
    }

    impl TestHttp {
        fn init() -> Self {
            Default::default()
        }
    }

    #[async_trait::async_trait]
    impl HttpIO for TestHttp {
        async fn execute(&self, request: reqwest::Request) -> Result<Response<Bytes>> {
            let response = self.client.execute(request).await;
            Response::from_reqwest(response?.error_for_status()?).await
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
        fn get(&self, key: &str) -> Option<String> {
            self.vars.get(key).cloned()
        }
    }

    impl TestEnvIO {
        pub fn init() -> Self {
            Self { vars: std::env::vars().collect() }
        }
    }

    pub fn init() -> TargetRuntime {
        let http: Arc<dyn HttpIO + Sync + Send> = Arc::new(TestHttp::init());

        let http2: Arc<dyn HttpIO + Sync + Send> = Arc::new(TestHttp::init());

        let file = TestFileIO::init();
        let env = TestEnvIO::init();

        TargetRuntime {
            http,
            http2_only: http2,
            env: Arc::new(env),
            file: Arc::new(file),
            cache: Arc::new(InMemoryCache::new()),
        }
    }
}

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
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    text_body: Option<String>,
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

#[derive(Debug, Default, Deserialize, Serialize, PartialEq)]
struct SDLError {
    message: String,
    trace: Vec<String>,
    description: Option<String>,
}

impl<'a> From<Cause<&'a str>> for SDLError {
    fn from(value: Cause<&'a str>) -> Self {
        SDLError {
            message: value.message.to_string(),
            trace: value.trace.iter().map(|e| e.to_string()).collect(),
            description: None,
        }
    }
}

impl From<Cause<String>> for SDLError {
    fn from(value: Cause<String>) -> Self {
        SDLError {
            message: value.message.to_string(),
            trace: value.trace.iter().map(|e| e.to_string()).collect(),
            description: value.description,
        }
    }
}

#[derive(Clone, Setters)]
struct ExecutionSpec {
    path: PathBuf,
    name: String,
    safe_name: String,

    server: Vec<(Source, String)>,
    mock: Option<Vec<Mock>>,
    env: Option<HashMap<String, String>>,
    assert: Option<Vec<APIRequest>>,
    files: BTreeMap<String, String>,

    // Annotations for the runner
    runner: Option<Annotation>,

    check_identity: bool,
    sdl_error: bool,
}

impl ExecutionSpec {
    async fn cargo_read(path: &str) -> anyhow::Result<Vec<ExecutionSpec>> {
        let dir_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(path)
            .canonicalize()?;
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
        INIT.call_once(|| {});

        let ast = markdown::to_mdast(&contents, &ParseOptions::default()).unwrap();
        let mut children = ast
            .children()
            .unwrap_or_else(|| panic!("Failed to parse {:?}: empty file unexpected", path))
            .iter()
            .peekable();

        let mut name: Option<String> = None;
        let mut server: Vec<(Source, String)> = Vec::with_capacity(2);
        let mut mock: Option<Vec<Mock>> = None;
        let mut env: Option<HashMap<String, String>> = None;
        let mut files: BTreeMap<String, String> = BTreeMap::new();
        let mut assert: Option<Vec<APIRequest>> = None;
        let mut runner: Option<Annotation> = None;
        let mut check_identity = false;
        let mut sdl_error = false;

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
                                        ));
                                    }
                                });
                            } else {
                                return Err(anyhow!(
                                    "Unexpected content of level 5 heading in {:?}: {:#?}",
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
                    } else if heading.depth == 6 {
                        if let Some(Node::Text(text)) = heading.children.first() {
                            match text.value.as_str() {
                                "check identity" => {
                                    check_identity = true;
                                }
                                "sdl error" => {
                                    sdl_error = true;
                                }
                                _ => {
                                    return Err(anyhow!(
                                        "Unexpected flag {:?} in {:?}",
                                        text.value,
                                        path,
                                    ));
                                }
                            };
                        } else {
                            return Err(anyhow!(
                                "Unexpected content of level 6 heading in {:?}: {:#?}",
                                path,
                                heading
                            ));
                        }
                    } else if heading.depth == 4 {
                        // Parse following code hblock
                        let (content, lang) = if let Some(Node::Code(code)) = children.next() {
                            (code.value.to_owned(), code.lang.to_owned())
                        } else {
                            return Err(anyhow!("Unexpected non-code block node or EOF after component definition in {:?}", path));
                        };

                        // Parse component name
                        if let Some(Node::Text(text)) = heading.children.first() {
                            let name = text.value.as_str();

                            if let Some(name) = name.strip_prefix("file:") {
                                if files.insert(name.to_string(), content).is_some() {
                                    return Err(anyhow!(
                                        "Double declaration of file {:?} in {:#?}",
                                        name,
                                        path
                                    ));
                                }
                            } else {
                                let lang = match lang {
                                    Some(x) => Ok(x),
                                    None => Err(anyhow!(
                                        "Unexpected languageless code block in {:?}",
                                        path
                                    )),
                                }?;

                                let source = Source::from_str(&lang)?;

                                match name {
                                    "server:" => {
                                        // Server configs are only parsed if the test isn't skipped.
                                        server.push((source, content));
                                    }
                                    "mock:" => {
                                        if mock.is_none() {
                                            mock = match source {
                                                Source::Json => Ok(serde_json::from_str(&content)?),
                                                Source::Yml => Ok(serde_yaml::from_str(&content)?),
                                                _ => Err(anyhow!("Unexpected language in mock block in {:?} (only JSON and YAML are supported)", path)),
                                            }?;
                                        } else {
                                            return Err(anyhow!("Unexpected number of mock blocks in {:?} (only one is allowed)", path));
                                        }
                                    }
                                    "env:" => {
                                        if env.is_none() {
                                            env = match source {
                                                Source::Json => Ok(serde_json::from_str(&content)?),
                                                Source::Yml => Ok(serde_yaml::from_str(&content)?),
                                                _ => Err(anyhow!("Unexpected language in env block in {:?} (only JSON and YAML are supported)", path)),
                                            }?;
                                        } else {
                                            return Err(anyhow!("Unexpected number of env blocks in {:?} (only one is allowed)", path));
                                        }
                                    }
                                    "assert:" => {
                                        if assert.is_none() {
                                            assert = match source {
                                                Source::Json => Ok(serde_json::from_str(&content)?),
                                                Source::Yml => Ok(serde_yaml::from_str(&content)?),
                                                _ => Err(anyhow!("Unexpected language in assert block in {:?} (only JSON and YAML are supported)", path)),
                                            }?;
                                        } else {
                                            return Err(anyhow!("Unexpected number of assert blocks in {:?} (only one is allowed)", path));
                                        }
                                    }
                                    _ => {
                                        return Err(anyhow!(
                                            "Unexpected component {:?} in {:?}: {:#?}",
                                            name,
                                            path,
                                            heading
                                        ));
                                    }
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
                "Unexpected blocks in {:?}: You must define a GraphQL Config in an execution test.",
                path,
            ));
        }

        let spec = ExecutionSpec {
            path: path.to_owned(),
            name: name.unwrap_or_else(|| path.file_name().unwrap().to_str().unwrap().to_string()),
            safe_name: path.file_name().unwrap().to_str().unwrap().to_string(),

            server,
            mock,
            env,
            assert,
            files,

            runner,
            check_identity,
            sdl_error,
        };

        anyhow::Ok(spec)
    }

    async fn server_context(
        &self,
        tailcall_executor: Result<TailcallExecutor, ValidationError<String>>,
        env: HashMap<String, String>,
    ) -> TailcallExecutor {
        let tailcall_executor = tailcall_executor.unwrap();
        let http = MockHttpClient::new(self.clone());
        let http = if let Some(script) = tailcall_executor.app_ctx.blueprint.server.script.clone() {
            javascript::init_http(http, script)
        } else {
            Arc::new(http)
        };

        let http2_only = if self.mock.is_some() {
            Arc::new(MockHttpClient::new(self.clone()))
        } else {
            http.clone()
        };

        let runtime = TargetRuntime {
            http,
            http2_only,
            file: Arc::new(MockFileSystem::new(self.clone())),
            env: Arc::new(Env::init(env)),
            cache: Arc::new(InMemoryCache::new()),
        };
        let mut tailcall_executor = tailcall_executor.clone();
        tailcall_executor.app_ctx = Arc::new(AppContext::new(
            tailcall_executor.app_ctx.blueprint.clone(),
            runtime,
        ));
        tailcall_executor.clone()
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
        let mocks = self.spec.mock.as_ref().unwrap();

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
                self.spec
                    .path
                    .strip_prefix(std::env::current_dir()?)
                    .unwrap_or(&self.spec.path)
                    .to_str()
                    .unwrap()
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
        if let Some(body) = mock_response.0.text_body {
            // Return plaintext body if specified
            let body = string_to_bytes(&body);
            response.body = Bytes::from_iter(body);
        } else if is_grpc {
            // Special Handling for GRPC
            let body = string_to_bytes(mock_response.0.body.as_str().unwrap());
            response.body = Bytes::from_iter(body);
        } else {
            let body = serde_json::to_vec(&mock_response.0.body)?;
            response.body = Bytes::from_iter(body);
        }

        Ok(response)
    }
}

struct MockFileSystem {
    spec: ExecutionSpec,
}

impl MockFileSystem {
    fn new(spec: ExecutionSpec) -> MockFileSystem {
        MockFileSystem { spec }
    }
}

#[async_trait::async_trait]
impl FileIO for MockFileSystem {
    async fn write<'a>(&'a self, _path: &'a str, _content: &'a [u8]) -> anyhow::Result<()> {
        Err(anyhow!("Cannot write to a file in an execution spec"))
    }

    async fn read<'a>(&'a self, path: &'a str) -> anyhow::Result<String> {
        let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/execution/");
        let path = path
            .strip_prefix(&base.to_string_lossy().to_string())
            .unwrap_or(path);

        match self.spec.files.get(path) {
            Some(x) => Ok(x.to_owned()),
            None => Err(anyhow!("No such file or directory (os error 2)")),
        }
    }
}

async fn assert_spec(spec: ExecutionSpec) {
    let will_insta_panic = std::env::var("INSTA_FORCE_PASS").is_err();

    // Parse and validate all server configs + check for identity
    log::info!("{} {} ...", spec.name, spec.path.display());

    if spec.sdl_error {
        // errors: errors are expected, make sure they match
        let (source, content) = &spec.server[0];

        if !matches!(source, Source::GraphQL) {
            panic!("Cannot use \"sdl error\" directive with a non-GraphQL server block.");
        }
        let mut runtime = test::init();
        runtime.file = Arc::new(MockFileSystem::new(spec.clone()));
        let tailcall_builder = TailcallBuilder::init(runtime);

        let tailcall_executor = tailcall_builder
            .with_config(
                Source::GraphQL,
                content,
                Some(spec.path.to_string_lossy().to_string()),
            )
            .await;

        match tailcall_executor {
            Ok(_) => {
                log::error!("\terror FAIL");
                panic!(
                    "Spec {} {:?} with \"sdl error\" directive did not have a validation error.",
                    spec.name, spec.path
                );
            }
            Err(cause) => {
                let errors: Vec<SDLError> =
                    cause.as_vec().iter().map(|e| e.to_owned().into()).collect();

                log::info!("\terrors... (snapshot)");

                let snapshot_name = format!("{}_errors", spec.safe_name);

                insta::assert_json_snapshot!(snapshot_name, errors);

                if will_insta_panic {
                    log::info!("\terrors ok");
                }
            }
        };

        return;
    }

    let mut server: Vec<Config> = Vec::with_capacity(spec.server.len());

    for (i, (source, content)) in spec.server.iter().enumerate() {
        let config = Config::from_source(source.to_owned(), content).unwrap_or_else(|e| {
            panic!(
                "Couldn't parse GraphQL in server definition #{} of {:#?}: {}",
                i + 1,
                spec.path,
                e
            )
        });

        let config = Config::default().merge_right(&config);

        log::info!("\tserver #{} parse ok", i + 1);

        // TODO: we should probably figure out a way to do this for every test
        // but GraphQL identity checking is very hard, since a lot depends on the code style
        // the re-serializing check gives us some of the advantages of the identity check too,
        // but we are missing out on some by having it only enabled for either new tests that request it
        // or old graphql_spec tests that were explicitly written with it in mind
        if spec.check_identity {
            if matches!(source, Source::GraphQL) {
                let identity = config.to_sdl();

                pretty_assertions::assert_eq!(
                    content.as_ref(),
                    identity,
                    "Identity check failed for {:#?}",
                    spec.path,
                );

                log::info!("\tserver #{} identity ok", i + 1);
            } else {
                panic!(
                    "Spec {:#?} has \"check identity\" enabled, but its config isn't in GraphQL.",
                    spec.path
                );
            }
        }

        server.push(config);
    }

    // merged: Run merged specs
    log::info!("\tmerged... (snapshot)");

    let merged = server
        .iter()
        .fold(Config::default(), |acc, c| acc.merge_right(c))
        .to_sdl();

    let snapshot_name = format!("{}_merged", spec.safe_name);

    insta::assert_snapshot!(snapshot_name, merged);

    if will_insta_panic {
        log::info!("\tmerged ok");
    }

    // Resolve all configs
    let mut runtime = test::init();
    runtime.file = Arc::new(MockFileSystem::new(spec.clone()));
    let tailcall_builder = TailcallBuilder::init(runtime);

    let server: Vec<Result<TailcallExecutor, ValidationError<String>>> = join_all(
        server // todo check
            .into_iter()
            .map(|config| {
                tailcall_builder.clone().with_config(
                    Source::GraphQL,
                    config.to_sdl(),
                    Some(spec.path.to_string_lossy().to_string()),
                )
            }),
    )
    .await
    .into_iter()
    .collect();

    if server.len() == 1 {
        let tailcall_executor = server.first().cloned().unwrap().unwrap();

        // client: Check if client spec matches snapshot
        let client = print_schema(tailcall_executor.app_ctx.blueprint.to_schema());
        let snapshot_name = format!("{}_client", spec.safe_name);

        log::info!("\tclient... (snapshot)");
        insta::assert_snapshot!(snapshot_name, client);

        if will_insta_panic {
            log::info!("\tclient ok");
        }
    }

    if let Some(assert_spec) = spec.assert.as_ref() {
        // assert: Run assert specs
        for (i, assertion) in assert_spec.iter().enumerate() {
            let response = run_assert(
                &spec,
                &spec
                    .env
                    .clone()
                    .unwrap_or_else(|| HashMap::with_capacity(0)),
                assertion,
                server.first().cloned().unwrap(),
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
                text_body: None,
            };

            let snapshot_name = format!("{}_assert_{}", spec.safe_name, i);

            log::info!("\tassert #{}... (snapshot)", i + 1);
            insta::assert_json_snapshot!(snapshot_name, response);

            if will_insta_panic {
                log::info!("\tassert #{} ok", i + 1);
            }
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .filter(Some("execution_spec"), log::LevelFilter::Info)
        .init();

    // Explicitly only run one test if specified in command line args
    // This is used by testconv to auto-apply the snapshots of unconvertable fail-annotated http specs
    let explicit = std::env::args().skip(1).find(|x| !x.starts_with("--"));
    let spec = if let Some(explicit) = explicit {
        let path = PathBuf::from(&explicit)
            .canonicalize()
            .unwrap_or_else(|_| panic!("Failed to parse explicit test path {:?}", explicit));

        let contents = fs::read_to_string(&path)?;
        let spec: ExecutionSpec = ExecutionSpec::from_source(&path, contents)
            .await
            .map_err(|err| err.context(path.to_str().unwrap().to_string()))?;

        vec![spec]
    } else {
        let spec = ExecutionSpec::cargo_read("tests/execution").await?;
        ExecutionSpec::filter_specs(spec)
    };

    for spec in spec.into_iter() {
        assert_spec(spec).await;
    }

    Ok(())
}

async fn run_assert(
    spec: &ExecutionSpec,
    env: &HashMap<String, String>,
    request: &APIRequest,
    tailcall_executor: Result<TailcallExecutor, ValidationError<String>>,
) -> anyhow::Result<hyper::Response<Body>> {
    let query_string = serde_json::to_string(&request.body).expect("body is required");
    let method = request.method.clone();
    let headers = request.headers.clone();
    let url = request.url.clone();
    let server_context = spec.server_context(tailcall_executor, env.clone()).await;
    let req = headers
        .into_iter()
        .fold(
            Request::builder()
                .method(method.to_hyper())
                .uri(url.as_str()),
            |acc, (key, value)| acc.header(key, value),
        )
        .body(Body::from(query_string))?;

    server_context.execute(req).await
}
