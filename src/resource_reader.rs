use std::collections::HashMap;

use futures_util::future::join_all;
use futures_util::TryFutureExt;
use lazy_static::lazy_static;
use tokio::sync::Mutex;
use url::Url;

use crate::runtime::TargetRuntime;

/// Response of a file read operation
#[derive(Debug)]
pub struct FileRead {
    pub content: String,
    pub path: String,
}

lazy_static! {
    // Global cache accessible by all instances of ResourceReader
    static ref CACHE: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

pub struct ResourceReader {
    runtime: TargetRuntime,
    enable_cache: bool,
}

impl ResourceReader {
    pub fn init(runtime: TargetRuntime, enable_cache: bool) -> Self {
        Self { runtime, enable_cache }
    }
    /// Reads a file from the filesystem or from an HTTP URL
    pub async fn read_file<T: ToString>(&self, file: T) -> anyhow::Result<FileRead> {
        let file_path = file.to_string();

        // Check cache first if caching is enabled
        if self.enable_cache {
            let cache = CACHE.lock().await;

            // If cache is found
            if let Some(content) = cache.get(&file_path) {
                return Ok(FileRead { content: content.clone(), path: file_path });
            }
        }

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

        // Save to cache if caching is enabled
        if self.enable_cache {
            let mut cache = CACHE.lock().await;
            cache.insert(file_path.clone(), content.clone());
        }

        Ok(FileRead { content, path: file.to_string() })
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
