
use std::path::Path;
use futures_util::{TryFutureExt, future::join_all};
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
}

impl ResourceReader {
    pub fn init(runtime: TargetRuntime) -> Self {
        Self { runtime }
    }
    /// Reads a file from the filesystem or from an HTTP URL
    pub async fn read_file<T: ToString>(&self, file: T) -> anyhow::Result<FileRead> {
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

    /// Checks if path is absolute else it joins file path with relative dir
    /// path
    pub fn resolve_path(src: &str, root_dir: Option<&Path>) -> String {
        if Path::new(&src).is_absolute() {
            src.to_string()
        } else {
            let path = root_dir.unwrap_or(Path::new(""));
            path.join(src).to_string_lossy().to_string()
        }
    }

}



#[cfg(test)]
mod resource_reader_tests {
    use std::path::{Path, PathBuf};
    use pretty_assertions::assert_eq;
    use crate::resource_reader::ResourceReader;
    
    #[test]
    fn test_relative_path() {
        let path_dir = Path::new("abc/xyz");
        let file_relative = "foo/bar/my.proto";
        let file_absolute = "/foo/bar/my.proto";
        assert_eq!(
            path_dir.to_path_buf().join(file_relative),
            PathBuf::from(ResourceReader::resolve_path(file_relative, Some(path_dir)))
        );
        assert_eq!(
            "/foo/bar/my.proto",
            ResourceReader::resolve_path(file_absolute, Some(path_dir))
        );
    }
}
