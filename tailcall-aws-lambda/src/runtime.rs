use std::borrow::Cow;
use std::sync::Arc;

use anyhow::anyhow;
use tailcall::core::cache::InMemoryCache;
use tailcall::core::runtime::TargetRuntime;
use tailcall::core::{EntityCache, EnvIO, FileIO};
use tokio::io::AsyncReadExt;

use crate::http::init_http;

#[derive(Clone, Copy)]
pub struct LambdaEnv;

impl EnvIO for LambdaEnv {
    fn get(&self, key: &str) -> Option<Cow<'_, str>> {
        // AWS Lambda sets environment variables
        // as real env vars in the runtime.
        std::env::var(key).ok().map(Cow::from)
    }
}

pub fn init_env() -> Arc<LambdaEnv> {
    Arc::new(LambdaEnv)
}

#[derive(Clone, Copy)]
pub struct LambdaFileIO;

#[async_trait::async_trait]
impl FileIO for LambdaFileIO {
    async fn write<'a>(&'a self, _path: &'a str, _content: &'a [u8]) -> anyhow::Result<()> {
        Err(anyhow!("File writing not supported on Lambda."))
    }

    async fn read<'a>(&'a self, path: &'a str) -> anyhow::Result<String> {
        let mut file = tokio::fs::File::open(path).await?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .await
            .map_err(anyhow::Error::from)?;
        Ok(String::from_utf8(buffer)?)
    }
}

pub fn init_file() -> Arc<LambdaFileIO> {
    Arc::new(LambdaFileIO)
}

pub fn init_cache() -> Arc<EntityCache> {
    Arc::new(InMemoryCache::new())
}

pub fn init_runtime() -> TargetRuntime {
    let http = init_http();
    TargetRuntime {
        http: http.clone(),
        http2_only: http,
        file: init_file(),
        env: init_env(),
        cache: init_cache(),
        extensions: Arc::new(vec![]),
        cmd_worker: None,
        worker: None,
    }
}
