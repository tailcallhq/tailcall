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

/// Supported Resources by Resource Reader
pub enum Resource {
    RawPath(String),
    Request(reqwest::Request),
}

impl From<reqwest::Request> for Resource {
    fn from(val: reqwest::Request) -> Self {
        Resource::Request(val)
    }
}

impl From<&str> for Resource {
    fn from(val: &str) -> Self {
        Resource::RawPath(val.to_owned())
    }
}

impl From<String> for Resource {
    fn from(val: String) -> Self {
        Resource::RawPath(val)
    }
}

#[async_trait::async_trait]
pub trait Reader {
    async fn read<T: Into<Resource> + ToString + Send>(&self, file: T) -> anyhow::Result<FileRead>;
}

#[derive(Clone)]
pub struct ResourceReader<A>(A);

impl<A: Reader + Send + Sync> ResourceReader<A> {
    pub async fn read_files<T>(&self, paths: &[T]) -> anyhow::Result<Vec<FileRead>>
    where
        T: Into<Resource> + Clone + ToString + Send,
    {
        let files = join_all(paths.iter().cloned().map(|path| {
            let path_str = path.to_string();
            self.read_file(path).map_err(|e| e.context(path_str))
        }))
        .await;

        files.into_iter().collect::<anyhow::Result<Vec<_>>>()
    }

    pub async fn read_file<T>(&self, path: T) -> anyhow::Result<FileRead>
    where
        T: Into<Resource> + ToString + Send,
    {
        self.0.read(path).await
    }
}

impl ResourceReader<Cached> {
    pub fn cached(runtime: TargetRuntime) -> Self {
        ResourceReader(Cached::init(runtime))
    }
}

impl std::fmt::Display for Resource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Resource::RawPath(file_path) => write!(f, "{}", file_path),
            Resource::Request(request_path) => write!(f, "{}", request_path.url()),
        }
    }
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
    async fn read<T: Into<Resource> + ToString + Send>(&self, file: T) -> anyhow::Result<FileRead> {
        let path = file.to_string();
        let content = match file.into() {
            Resource::RawPath(file_path) => {
                // Is an HTTP URL

                if let Ok(url) = Url::parse(&file_path) {
                    if url.scheme().starts_with("http") {
                        let response = self
                            .runtime
                            .http
                            .execute(reqwest::Request::new(reqwest::Method::GET, url))
                            .await?;

                        String::from_utf8(response.body.to_vec())?
                    } else {
                        // Is a file path on Windows
                        self.runtime.file.read(&file_path).await?
                    }
                } else {
                    // Is a file path
                    self.runtime.file.read(&file_path).await?
                }
            }
            Resource::Request(request) => {
                let response = self.runtime.http.execute(request).await?;
                String::from_utf8(response.body.to_vec())?
            }
        };
        Ok(FileRead { content, path })
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
    async fn read<T: Into<Resource> + ToString + Send>(&self, file: T) -> anyhow::Result<FileRead> {
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
            let file_read = self.direct.read(file).await?;
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
