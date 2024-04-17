use std::collections::HashMap;
use std::sync::Arc;

use async_lock::Mutex;
use futures_util::future::join_all;
use futures_util::TryFutureExt;
use url::Url;

use crate::runtime::TargetRuntime;

/// Response of a file read operation
#[derive(Debug)]
pub struct FileRead {
    pub content: String,
    pub path: String,
}

pub struct ResourceReader {
    runtime: TargetRuntime,
    // Cache file content, path->content
    cache: Option<Arc<Mutex<HashMap<String, String>>>>,
}

impl ResourceReader {
    pub fn init(runtime: TargetRuntime, cache: bool) -> Self {
        if cache {
            Self { runtime, cache: Some(Default::default()) }
        } else {
            Self { runtime, cache: None }
        }
    }

    pub async fn read_file<T: ToString>(&self, file: T) -> anyhow::Result<FileRead> {
        if self.cache.is_some() {
            self.read_file_with_cached(file).await
        } else {
            self.read_file_direct(file).await
        }
    }

    /// Reads a file from the filesystem or from an HTTP URL with cache
    async fn read_file_with_cached<T: ToString>(&self, file: T) -> anyhow::Result<FileRead> {
        // check cache
        let file_path = file.to_string();
        let content = self
            .cache
            .as_ref()
            .unwrap()
            .lock()
            .await
            .get(&file_path)
            .map(|v| v.to_owned());
        let content = if let Some(content) = content {
            content.to_owned()
        } else {
            let file_read = self.read_file_direct(file.to_string()).await?;
            self.cache
                .as_ref()
                .unwrap()
                .lock()
                .await
                .insert(file_path.to_owned(), file_read.content.clone());
            file_read.content
        };

        Ok(FileRead { content, path: file_path })
    }

    /// Reads a file from the filesystem or from an HTTP URL
    async fn read_file_direct<T: ToString>(&self, file: T) -> anyhow::Result<FileRead> {
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

    pub async fn read_files<T: ToString>(&self, files: &[T]) -> anyhow::Result<Vec<FileRead>> {
        if self.cache.is_some() {
            self.read_files_with_cached(files).await
        } else {
            self.read_files_direct(files).await
        }
    }

    /// Reads all the files in parallel
    async fn read_files_direct<T: ToString>(&self, files: &[T]) -> anyhow::Result<Vec<FileRead>> {
        let files = files.iter().map(|x| {
            self.read_file_direct(x.to_string())
                .map_err(|e| e.context(x.to_string()))
        });
        let content = join_all(files)
            .await
            .into_iter()
            .collect::<anyhow::Result<Vec<_>>>()?;
        Ok(content)
    }

    /// Reads all the files in parallel with cache
    async fn read_files_with_cached<T: ToString>(
        &self,
        files: &[T],
    ) -> anyhow::Result<Vec<FileRead>> {
        let files = files.iter().map(|x| {
            self.read_file_with_cached(x.to_string())
                .map_err(|e| e.context(x.to_string()))
        });
        let content = join_all(files)
            .await
            .into_iter()
            .collect::<anyhow::Result<Vec<_>>>()?;
        Ok(content)
    }
}
