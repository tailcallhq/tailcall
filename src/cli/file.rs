use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::io::FileIO;

pub struct NativeFileIO {}

impl NativeFileIO {
  pub fn init() -> Self {
    NativeFileIO {}
  }
}

impl FileIO for NativeFileIO {
  async fn write<'a>(&'a self, file: &'a str, content: &'a [u8]) -> Result<()> {
    let mut file = tokio::fs::File::create(file).await?;
    file.write_all(content).await?;
    Ok(())
  }

  async fn read<'a>(&'a self, file_path: &'a str) -> Result<String> {
    let mut file = tokio::fs::File::open(file_path).await?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).await?;
    Ok(String::from_utf8(buffer)?)
  }
}
