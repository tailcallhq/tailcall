use anyhow::Result;
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
  async fn write<'a>(&'a self, file_path: &'a str, content: &'a [u8]) -> Result<()> {
    let mut file = tokio::fs::File::create(file_path).await?;
    file.write_all(content).await.map_err(CLIError::from)?;
    log::info!("File write: {} ... ok", file_path);
    Ok(())
  }

  async fn read<'a>(&'a self, file_path: &'a str) -> Result<String> {
    let mut file = tokio::fs::File::open(file_path).await?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).await.map_err(CLIError::from)?;
    log::info!("File read: {} ... ok", file_path);
    Ok(String::from_utf8(buffer)?)
  }
}
