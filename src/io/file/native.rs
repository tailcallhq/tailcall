use anyhow::{anyhow, Result};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use url::Url;

use super::FileIO;

pub struct NativeFileIO {}

impl NativeFileIO {
  pub fn init() -> Self {
    NativeFileIO {}
  }
}

#[async_trait::async_trait]
impl FileIO for NativeFileIO {
  async fn write<'a>(file: &'a str, content: &'a [u8]) -> Result<()> {
    let mut file = tokio::fs::File::create(file).await?;
    file.write_all(content).await?;
    Ok(())
  }

  async fn read_file(file_path: &str) -> Result<(String, String)> {
    if let Ok(url) = Url::parse(file_path) {
      let response = crate::io::http::get_string(url).await?;
      let sdl = response.headers.get("content-type");
      let sdl = match sdl {
        Some(v) => v.to_str().map_err(|e| anyhow!("{}", e))?.to_string(),
        None => file_path.to_string(),
      };
      return Ok((response.body, sdl));
    }
    let mut file = tokio::fs::File::open(file_path).await?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).await?;
    Ok((String::from_utf8(buffer)?, file_path.to_string()))
  }

  async fn read_files<'a>(&'a self, file_paths: &'a [String]) -> Result<Vec<(String, String)>> {
    let mut files = vec![];
    for file in file_paths {
      let content = Self::read_file(file).await?;
      files.push(content);
    }
    Ok(files)
  }
}
