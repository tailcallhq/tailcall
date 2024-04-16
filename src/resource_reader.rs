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
    cache: Arc<Mutex<HashMap<String, String>>>,
}

impl ResourceReader {
    pub fn init(
        runtime: TargetRuntime,
        cache: Arc<Mutex<HashMap<String, String>>>,
    ) -> Self {
        Self { runtime, cache }
    }
    /// Reads a file from the filesystem or from an HTTP URL
    pub async fn read_file<T: ToString>(&self, file: T) -> anyhow::Result<FileRead> {
        // check cache
        let file_path = file.to_string();
        let content = self
            .cache
            .lock()
            .await
            .get(&file_path)
            .map(|v| v.to_owned());
        let content = if let Some(content) = content {
            content
        } else {
            let content = self.do_read_file(file.to_string()).await?;
            self.cache
                .lock()
                .await
                .insert(file_path.to_owned(), content.clone());
            content
        };

        Ok(FileRead { content, path: file_path })
    }

    async fn do_read_file<T: ToString>(&self, file: T) -> anyhow::Result<String> {
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
        Ok(content)
    }

    /// Reads all the files in parallel
    pub async fn read_files<T: ToString>(&self, files: &[T]) -> anyhow::Result<Vec<FileRead>> {
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
}
