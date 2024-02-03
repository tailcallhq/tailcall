use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Context;
use async_trait::async_trait;
use hyper::body::Bytes;
use lazy_static::lazy_static;
use reqwest::{Client, Request};

use crate::cache::InMemoryCache;
use crate::http::Response;
use crate::target_runtime::TargetRuntime;
use crate::{EnvIO, HttpIO};

lazy_static! {
    static ref FILES: HashMap<String, String> = {
        let mut m = HashMap::new();
        m.insert(
            "src/grpc/tests/news.proto".to_string(),
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/grpc/tests/news.proto"
            ))
            .to_string(),
        );
        m.insert(
            "src/grpc/tests/greetings.proto".to_string(),
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/grpc/tests/greetings.proto"
            ))
            .to_string(),
        );
        m.insert(
            "src/grpc/tests/nested0.proto".to_string(),
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/grpc/tests/nested0.proto"
            ))
            .to_string(),
        );
        m.insert(
            "src/grpc/tests/nested1.proto".to_string(),
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/grpc/tests/nested1.proto"
            ))
            .to_string(),
        );
        m.insert(
            "src/grpc/tests/cycle.proto".to_string(),
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/grpc/tests/cycle.proto"
            ))
            .to_string(),
        );
        m.insert(
            "src/grpc/tests/duplicate.proto".to_string(),
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/grpc/tests/duplicate.proto"
            ))
            .to_string(),
        );
        m.insert(
            "examples/jsonplaceholder.graphql".to_string(),
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/examples/jsonplaceholder.graphql"
            ))
            .to_string(),
        );
        m.insert(
            "examples/jsonplaceholder.json".to_string(),
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/examples/jsonplaceholder.json"
            ))
            .to_string(),
        );
        m.insert(
            "examples/jsonplaceholder.yml".to_string(),
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/examples/jsonplaceholder.yml"
            ))
            .to_string(),
        );
        m.insert(
            "examples/jsonplaceholder_script.graphql".to_string(),
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/examples/jsonplaceholder_script.graphql"
            ))
            .to_string(),
        );
        m.insert(
            "examples/scripts/echo.js".to_string(),
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/examples/scripts/echo.js"
            ))
            .to_string(),
        );
        m
    };
}

pub struct Env {
    env: HashMap<String, String>,
}

#[derive(Clone)]
pub struct FileIO {}

impl FileIO {
    pub fn init() -> Self {
        FileIO {}
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
        let content = FILES.get(&path).context("No such file")?;
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
