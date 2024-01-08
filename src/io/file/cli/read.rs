use tokio::fs::File;
use tokio::io::AsyncReadExt;
use anyhow::{anyhow, Result};
use url::Url;
use crate::io::file::FileIO;

impl FileIO {
    pub async fn read_file(file_path: &str) -> Result<(String,String)> {
        if let Ok(url) = Url::parse(file_path) {
            let response = crate::io::http::get_string(url).await?;
            let sdl = response.headers.get("content-type");
            let sdl = match sdl {
                Some(v) => v.to_str().map_err(|e|anyhow!("{}",e))?.to_string(),
                None => file_path.to_string(),
            };
            return Ok((response.body, sdl));
        }
        let mut f = File::open(file_path).await?;
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer).await?;
        Ok((String::from_utf8(buffer)?, file_path.to_string()))
    }

    pub async fn read_files(&self) -> Result<Vec<(String, String)>> {
        let mut files = vec![];
        for file in &self.files {
            let content = Self::read_file(file).await?;
            files.push(content);
        }
        Ok(files)
    }
}
