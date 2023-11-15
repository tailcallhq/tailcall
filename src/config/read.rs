use tokio::fs::File;
use tokio::io::AsyncReadExt;
use crate::config::{Config, Source};

pub struct ConfigReader {
    config: Config,
}

impl ConfigReader {
    pub fn init() -> Self {
        Self { config: Config::default() }
    }
    pub async fn serialize_config(&mut self, path: &String) -> anyhow::Result<()> {
        let conf = if path.starts_with("http://") || path.starts_with("https://") {
            let server_sdl = Self::read_over_url(path).await?;
            Config::from_source(Source::try_parse_and_detect(&server_sdl)?, &server_sdl)? // needs improvement
        } else {
            Config::from_file_path(path).await?
        };
        self.config = self.config.clone().merge_right(&conf);
        Ok(())
    }
    pub async fn read_file(file_path: &String) -> anyhow::Result<String> {
        let mut f = File::open(file_path).await?;
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer).await?;
        Ok(String::from_utf8(buffer)?)
    }
    async fn read_over_url(path: &String) -> anyhow::Result<String> {
        let resp = reqwest::get(path).await?;
        Ok(resp.text().await?)
    }
    pub fn get_config(&self) -> &Config {
        &self.config
    }
}