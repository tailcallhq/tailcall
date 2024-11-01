use miette::IntoDiagnostic;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::core::FileIO;

#[derive(Clone)]
pub struct NativeFileIO {}

impl NativeFileIO {
    pub fn init() -> Self {
        NativeFileIO {}
    }
}

async fn read(path: &str) -> miette::Result<String> {
    let mut file = tokio::fs::File::open(path).await.into_diagnostic()?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).await.into_diagnostic()?;
    String::from_utf8(buffer).into_diagnostic()
}

async fn write<'a>(path: &'a str, content: &'a [u8]) -> miette::Result<()> {
    let mut file = tokio::fs::File::create(path).await.into_diagnostic()?;
    file.write_all(content).await.into_diagnostic()?;
    Ok(())
}

#[async_trait::async_trait]
impl FileIO for NativeFileIO {
    async fn write<'a>(&'a self, path: &'a str, content: &'a [u8]) -> miette::Result<()> {
        write(path, content).await?;
        tracing::info!("File write: {} ... ok", path);
        Ok(())
    }

    async fn read<'a>(&'a self, path: &'a str) -> miette::Result<String> {
        let content = read(path).await?;
        tracing::info!("File read: {} ... ok", path);
        Ok(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_write_error() {
        // Attempt to write to an invalid path
        let file_io = NativeFileIO::init();
        let result = file_io.write("/invalid/path/to/file.txt", b"content").await;

        // Verify that an error is returned
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_read_error() {
        // Attempt to read from a non-existent file
        let file_io = NativeFileIO::init();
        let result = file_io.read("/non/existent/file.txt").await;

        // Verify that an error is returned
        assert!(result.is_err());
    }
}
