use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use futures_util::future::join_all;
use futures_util::TryFutureExt;

use crate::core::runtime::TargetRuntime;

/// Response of a file read operation
#[derive(Debug)]
pub struct FileRead {
    pub content: String,
    pub path: String,
}

/// Supported Resources by Resource Reader
enum Resource {
    File(String),
    Request(reqwest::Request),
}

#[async_trait::async_trait]
pub trait Reader {
    async fn read<T: Into<Resource> + ToString + Send>(&self, file: T) -> anyhow::Result<FileRead>;
}

#[derive(Clone)]
pub struct ResourceReader<A>(A);

impl<A: Reader + Send + Sync + 'static> ResourceReader<A> {
    pub async fn read_files(&self, paths: Vec<Resource>) -> anyhow::Result<Vec<FileRead>> {
        let files: Vec<_> = paths
            .into_iter()
            .map(|path| {
                let path_str = path.to_string();
                self.read_file(path).map_err(|e| e.context(path_str))
            })
            .collect();

        let content = join_all(files)
            .await
            .into_iter()
            .collect::<anyhow::Result<Vec<_>>>()?;
        Ok(content)
    }

    pub async fn read_file(&self, path: Resource) -> anyhow::Result<FileRead> {
        self.0.read(path).await
    }
}

impl ResourceReader<Cached> {
    pub fn cached(runtime: TargetRuntime) -> Self {
        ResourceReader(Cached::init(runtime))
    }
}

impl ToString for Resource {
    fn to_string(&self) -> String {
        match self {
            Resource::File(file_path) => file_path.to_string(),
            Resource::Request(request_path) => request_path.url().to_string(),
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
            Resource::File(file_path) => self.runtime.file.read(&file_path).await?,
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
