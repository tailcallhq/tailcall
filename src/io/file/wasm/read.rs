use anyhow::{anyhow, Result};
use url::Url;
use crate::io::file::FileIO;

impl FileIO {
    pub async fn read_file(file_path: &str) -> Result<(String,String)> {
        let url =  Url::parse(file_path)?;
        let response = crate::io::http::get_string(url).await?;
        let sdl = response.headers.get("Content-Type");
        let sdl = match sdl {
            Some(v) => v.to_str().map_err(|e|anyhow!("{}",e))?.to_string(),
            None => file_path.to_string(),
        };
        Ok((response.body, sdl))
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
