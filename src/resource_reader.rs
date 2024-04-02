use futures_util::future::join_all;
use futures_util::TryFutureExt;
use url::Url;

use crate::runtime::TargetRuntime;

/// Response of a file read operation
#[derive(Debug)]
pub struct FileRead {
    pub content: String,
    pub path: String,
    pub content_ty: Option<String>,
}

pub struct ResourceReader {
    runtime: TargetRuntime,
}

impl ResourceReader {
    pub fn init(runtime: TargetRuntime) -> Self {
        Self { runtime }
    }
    /// Reads a file from the filesystem or from an HTTP URL
    pub async fn read_file<T: ToString>(&self, file: T) -> anyhow::Result<FileRead> {
        // Is an HTTP URL
        let (content, content_ty) = if let Ok(url) = Url::parse(&file.to_string()) {
            if url.scheme().starts_with("http") {
                let response = self
                    .runtime
                    .http
                    .execute(reqwest::Request::new(reqwest::Method::GET, url))
                    .await?;
                let content_ty = response
                    .headers
                    .get("Content-Type")
                    .and_then(|v| v.to_str().map(|v| v.to_string()).ok());

                (String::from_utf8(response.body.to_vec())?, content_ty)
            } else {
                // Is a file path on Windows

                (self.runtime.file.read(&file.to_string()).await?, None)
            }
        } else {
            // Is a file path

            (self.runtime.file.read(&file.to_string()).await?, None)
        };

        Ok(FileRead { content, path: file.to_string(), content_ty })
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
