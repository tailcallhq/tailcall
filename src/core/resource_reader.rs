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
    async fn read<T: Into<Resource> + Send>(&self, file: T) -> anyhow::Result<FileRead>;
}

#[derive(Clone)]
pub struct ResourceReader<A>(A);

impl<A: Reader + Send + Sync> ResourceReader<A> {
    pub async fn read_files<T>(&self, paths: &[T]) -> anyhow::Result<Vec<FileRead>>
    where
        T: Into<Resource> + Clone + Send,
    {
        let files = join_all(paths.iter().cloned().map(|path| {
            let resource: Resource = path.into();
            let resource_path = resource.to_string();
            self.read_file(resource)
                .map_err(|e| e.context(resource_path))
        }))
        .await;

        files.into_iter().collect::<anyhow::Result<Vec<_>>>()
    }

    pub async fn read_file<T>(&self, path: T) -> anyhow::Result<FileRead>
    where
        T: Into<Resource> + Send,
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
    async fn read<T: Into<Resource> + Send>(&self, file: T) -> anyhow::Result<FileRead> {
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

                        let content = String::from_utf8(response.body.to_vec())?;
                        FileRead { path: file_path, content }
                    } else {
                        // Is a file path on Windows
                        let content = self.runtime.file.read(&file_path).await?;
                        FileRead { path: file_path, content }
                    }
                } else {
                    // Is a file path
                    let content = self.runtime.file.read(&file_path).await?;
                    FileRead { path: file_path, content }
                }
            }
            Resource::Request(request) => {
                let request_url = request.url().to_string();
                let response = self.runtime.http.execute(request).await?;
                let content = String::from_utf8(response.body.to_vec())?;

                FileRead { path: request_url, content }
            }
        };
        Ok(content)
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
    async fn read<T: Into<Resource> + Send>(&self, file: T) -> anyhow::Result<FileRead> {
        // check cache
        let resource: Resource = file.into();
        let file_path = resource.to_string();
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
            let file_read = self.direct.read(resource).await?;
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

#[cfg(test)]
mod test {
    use super::*;

    impl Resource {
        fn as_request(&self) -> Option<&reqwest::Request> {
            match self {
                Resource::Request(request) => Some(request),
                _ => None,
            }
        }

        fn as_raw_path(&self) -> Option<&str> {
            match self {
                Resource::RawPath(path) => Some(path),
                _ => None,
            }
        }
    }

    #[test]
    fn test_from_reqwest_request() {
        let original_url: Url = "https://tailcall.run".parse().unwrap();
        let original_request = reqwest::Request::new(reqwest::Method::GET, original_url.clone());
        let resource: Resource = original_request.try_clone().unwrap().into();
        let request = resource.as_request().unwrap();

        let actual = request;
        let expected = original_request;

        assert_eq!(actual.method(), expected.method());
        assert_eq!(actual.url(), expected.url());
    }

    #[test]
    fn test_from_str() {
        let path = "https://tailcall.run";
        let resource: Resource = path.into();

        let actual = resource.as_raw_path().unwrap();
        let expected = path;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_from_string() {
        let path = String::from("./config.graphql");
        let resource: Resource = path.clone().into();
        let actual = resource.as_raw_path().unwrap();
        let expected = path.as_str();

        assert_eq!(actual, expected);
    }
}
