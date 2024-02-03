use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Context;
use async_trait::async_trait;
use hyper::body::Bytes;
use reqwest::{Client, Request};

use crate::cache::InMemoryCache;
use crate::http::Response;
use crate::target_runtime::TargetRuntime;
use crate::{EnvIO, HttpIO};

macro_rules! include_file {
    ($name:literal) => {
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), $name))
    };
}

const NESTED0: &str = include_file!("/src/grpc/tests/nested0.proto");
const NESTED1: &str = include_file!("/src/grpc/tests/nested1.proto");
const GREETINGS: &str = include_file!("/src/grpc/tests/greetings.proto");
const NEWS: &str = include_file!("/src/grpc/tests/news.proto");
const CYCLE: &str = include_file!("/src/grpc/tests/cycle.proto");
const DUPLICATE: &str = include_file!("/src/grpc/tests/duplicate.proto");
const JSONPLACEHOLDER_JSON: &str = include_file!("/examples/jsonplaceholder.json");
const JSONPLACEHOLDER_YML: &str = include_file!("/examples/jsonplaceholder.yml");
const JSONPLACEHOLDER_GQL: &str = include_file!("/examples/jsonplaceholder.graphql");
const JSONPLACEHOLDER_SCRIPT: &str = include_file!("/examples/jsonplaceholder_script.graphql");
const ECHO_JS: &str = include_file!("/examples/scripts/echo.js");

pub struct Env {
    env: HashMap<String, String>,
}

#[derive(Clone)]
pub struct FileIO {
    files: HashMap<String, String>,
}

impl FileIO {
    pub fn init() -> Self {
        let mut files = HashMap::new();
        files.insert("src/grpc/tests/news.proto".into(), NEWS.into());
        files.insert("src/grpc/tests/greetings.proto".into(), GREETINGS.into());
        files.insert("src/grpc/tests/nested0.proto".into(), NESTED0.into());
        files.insert("src/grpc/tests/nested1.proto".into(), NESTED1.into());
        files.insert("src/grpc/tests/cycle.proto".into(), CYCLE.into());
        files.insert("src/grpc/tests/duplicate.proto".into(), DUPLICATE.into());

        files.insert(
            "examples/jsonplaceholder.graphql".into(),
            JSONPLACEHOLDER_GQL.into(),
        );
        files.insert(
            "examples/jsonplaceholder.json".into(),
            JSONPLACEHOLDER_JSON.into(),
        );
        files.insert(
            "examples/jsonplaceholder.yml".into(),
            JSONPLACEHOLDER_YML.into(),
        );
        files.insert(
            "examples/jsonplaceholder_script.graphql".into(),
            JSONPLACEHOLDER_SCRIPT.into(),
        );
        files.insert("examples/scripts/echo.js".into(), ECHO_JS.into());

        FileIO { files }
    }
}

#[async_trait::async_trait]
impl crate::FileIO for FileIO {
    async fn write<'a>(&'a self, path: &'a str, _: &'a [u8]) -> anyhow::Result<()> {
        // *self.files.get_mut(path) = ;
        log::info!("File write: {} ... ok", path);
        Ok(())
    }

    async fn read<'a>(&'a self, path: &'a str) -> anyhow::Result<String> {
        let path = path_to_file_name(Path::new(path)).unwrap_or(path.to_string());
        let content = self.files.get(&path).context("No such file")?;
        log::info!("File read: {} ... ok", path);
        Ok(content.clone())
    }
}

pub fn path_to_file_name(path: &Path) -> Option<String> {
    let components: Vec<_> = path.components().collect();

    // Find the index of the "src" component
    if let Some(src_index) = components.iter().position(|&c| c.as_os_str() == "src") {
        // Reconstruct the path from the "src" component onwards
        let after_src_components = &components[src_index..];
        let result = after_src_components
            .iter()
            .fold(PathBuf::new(), |mut acc, comp| {
                acc.push(comp);
                acc
            });
        Some(result.to_str().unwrap().to_string())
    } else {
        None
    }
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

struct Http {
    client: Client,
}

#[async_trait]
impl HttpIO for Http {
    async fn execute(&self, request: Request) -> anyhow::Result<Response<Bytes>> {
        let client = self.client.clone();
        let fx = async move {
            let response = client.execute(request).await?.error_for_status()?;
            Response::from_reqwest(response).await
        };
        #[cfg(target_arch = "wasm32")]
        let res = async_std::task::spawn_local(fx).await?;
        #[cfg(not(target_arch = "wasm32"))]
        let res = fx.await?;
        Ok(res)
    }
}

pub fn init_test_runtime() -> TargetRuntime {
    let http = Arc::new(Http { client: Client::new() });
    let http2_only = http.clone();
    TargetRuntime {
        http,
        http2_only,
        env: Arc::new(Env::init(HashMap::new())),
        file: Arc::new(FileIO::init()),
        cache: Arc::new(InMemoryCache::new()),
    }
}
