use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::cli::CLIError;
use crate::FileIO;

#[derive(Clone)]
pub struct TestFileIO {}

impl TestFileIO {
    pub fn init() -> Self {
        TestFileIO {}
    }
}

#[async_trait::async_trait]
impl FileIO for TestFileIO {
    async fn write<'a>(&'a self, path: &'a str, content: &'a [u8]) -> anyhow::Result<()> {
        let mut file = tokio::fs::File::create(path).await?;
        file.write_all(content).await.map_err(CLIError::from)?;
        Ok(())
    }

    async fn read<'a>(&'a self, path: &'a str) -> anyhow::Result<String> {
        let mut file = tokio::fs::File::open(path).await?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .await
            .map_err(CLIError::from)?;
        Ok(String::from_utf8(buffer)?)
    }
}
