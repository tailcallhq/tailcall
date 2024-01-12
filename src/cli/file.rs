use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::io::FileIO;

pub struct NativeFileIO {}

impl NativeFileIO {
  pub fn init() -> Self {
    NativeFileIO {}
  }
}

#[async_trait::async_trait]
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

  async fn read_all<'a>(&'a self, file_paths: &'a [String]) -> Result<Vec<(String, String)>> {
    let mut files = vec![];
    // TODO: make this parallel
    for file in file_paths {
      let content = self.read(file).await?;
      files.push((content, file.to_string()));
    }
    Ok(files)
  }
}
