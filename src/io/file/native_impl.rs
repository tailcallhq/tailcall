use anyhow::{anyhow, Result};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use url::Url;

use crate::io::file::FileIO;

impl FileIO {
  pub async fn write(file: &str, content: &[u8]) -> Result<()> {
    let mut file = tokio::fs::File::create(file).await?;
    file.write_all(content).await?;
    Ok(())
  }

  pub async fn read_file(file_path: &str) -> Result<(String, String)> {
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

  pub async fn read_files(&self, src_files: &Vec<String>) -> Result<Vec<(String, String)>> {
    let mut files = vec![];
    for file in src_files {
      let content = Self::read_file(file).await?;
      files.push(content);
    }
    Ok(files)
  }
}
