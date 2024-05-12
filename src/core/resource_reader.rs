use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use futures_util::future::join_all;
use futures_util::TryFutureExt;
use url::Url;

use crate::core::runtime::TargetRuntime;

/// Response of a file read operation
#[derive(Debug)]
pub struct FileRead {
    pub content: String,
    pub path: String,
}

#[derive(Clone)]
pub struct ResourceReader<A>(A);

impl<A: Reader + Send + Sync> ResourceReader<A> {
    /// Reads all the files in parallel
    pub async fn read_files<T: ToString + Send + Sync>(
        &self,
        files: &[T],
    ) -> anyhow::Result<Vec<FileRead>> {
        let files = files.iter().map(|x| {
            self.read_file(x.to_string())
                .map_err(|e| e.context(x.to_string()))
        });
        let content = join_all(files)
            .await
            .into_iter()
            .collect::<anyhow::Result<Vec<_>>>()?;
        Ok(content)
    }

    pub async fn read_file<T: ToString + Send>(&self, file: T) -> anyhow::Result<FileRead> {
        self.0.read(file).await
    }
}

impl ResourceReader<Cached> {
    pub fn cached(runtime: TargetRuntime) -> Self {
        ResourceReader(Cached::init(runtime))
    }
}

#[async_trait::async_trait]
pub trait Reader {
    async fn read<T: ToString + Send>(&self, file: T) -> anyhow::Result<FileRead>;
}

/// Reads the files directly from the filesystem or from an HTTP URL
#[derive(Clone)]
pub struct Direct {
    runtime: TargetRuntime,
}

impl Direct {
    pub fn init(runtime: TargetRuntime) -> Self {
        Self { runtime }
    }
}

#[async_trait::async_trait]
impl Reader for Direct {
    /// Reads a file from the filesystem or from an HTTP URL
    async fn read<T: ToString + Send>(&self, file: T) -> anyhow::Result<FileRead> {
        // Is an HTTP URL
        let content = if let Ok(url) = Url::parse(&file.to_string()) {
            if url.scheme().starts_with("http") {
                let response = self
                    .runtime
                    .http
                    .execute(reqwest::Request::new(reqwest::Method::GET, url))
                    .await?;

                String::from_utf8(response.body.to_vec())?
            } else {
                // Is a file path on Windows

                self.runtime.file.read(&file.to_string()).await?
            }
        } else {
            // Is a file path

            self.runtime.file.read(&file.to_string()).await?
        };
        Ok(FileRead { content, path: file.to_string() })
    }
}

/// Reads the files from the filesystem or from an HTTP URL with cache
#[derive(Clone)]
pub struct Cached {
    direct: Direct,
    // Cache file content, path -> content
    cache: Arc<Mutex<HashMap<String, String>>>,
}

impl Cached {
    pub fn init(runtime: TargetRuntime) -> Self {
        Self { direct: Direct::init(runtime), cache: Default::default() }
    }
}

#[async_trait::async_trait]
impl Reader for Cached {
    /// Reads a file from the filesystem or from an HTTP URL with cache
    async fn read<T: ToString + Send>(&self, file: T) -> anyhow::Result<FileRead> {
        // check cache
        let file_path = file.to_string();
        let content = self
            .cache
            .as_ref()
            .lock()
            .unwrap()
            .get(&file_path)
            .map(|v| v.to_owned());
        let content = if let Some(content) = content {
            content.to_owned()
        } else {
            let file_read = self.direct.read(file.to_string()).await?;
            self.cache
                .as_ref()
                .lock()
                .unwrap()
                .insert(file_path.to_owned(), file_read.content.clone());
            file_read.content
        };

        Ok(FileRead { content, path: file_path })
    }
}
