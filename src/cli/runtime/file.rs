use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::cli::CLIError;
use crate::FileIO;

#[derive(Clone)]
pub struct NativeFileIO {}

impl NativeFileIO {
    pub fn init() -> Self {
        NativeFileIO {}
    }
}

#[async_trait::async_trait]
impl FileIO for NativeFileIO {
    async fn write<'a>(&'a self, path: &'a str, content: &'a [u8]) -> anyhow::Result<()> {
        let mut file = tokio::fs::File::create(path).await?;
        file.write_all(content).await.map_err(CLIError::from)?;
        tracing::info!("File write: {} ... ok", path);
        Ok(())
    }

    async fn read<'a>(&'a self, path: &'a str) -> anyhow::Result<String> {
        let mut file = tokio::fs::File::open(path).await?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .await
            .map_err(CLIError::from)?;
        tracing::info!("File read: {} ... ok", path);
        Ok(String::from_utf8(buffer)?)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::NamedTempFile;

    use super::*;

    #[tokio::test]
    async fn test_write_and_read_file() {
        // Setup - Create a temporary file
        let tmp_file = NamedTempFile::new().expect("Failed to create temp file");
        let tmp_path = tmp_file
            .path()
            .to_str()
            .expect("Failed to get temp file path");
        let file_io = NativeFileIO::init();

        // Test writing to the file
        let content = b"Hello, world!";
        file_io
            .write(tmp_path, content)
            .await
            .expect("Failed to write to temp file");

        // Test reading from the file
        let read_content = file_io
            .read(tmp_path)
            .await
            .expect("Failed to read from temp file");

        // Verify the content is as expected
        assert_eq!(read_content, String::from_utf8_lossy(content));
    }

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
