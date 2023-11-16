use std::slice::Iter;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

use crate::config::{Config, Source};

pub struct ConfigReader {
  config: Config,
  file_paths: Iter<'_,String>
}

impl ConfigReader {
  pub fn init(file_paths: Iter<String>) -> Self {
    Self { config: Config::default(), file_paths }
  }
  pub async fn read(&mut self) -> anyhow::Result<Config> {
    for path in self.file_paths {
      let conf = if let Ok(url) = reqwest::Url::parse(path) {
        let (st, source) = Self::read_over_url(url).await?;
        Config::from_source(source, &st)?
      } else {
        let path = path.trim_end_matches('/');
        Self::from_file_path(path).await?
      };
      self.config = self.config.clone().merge_right(&conf);
    }
    Ok(self.config.clone())
  }
  pub async fn from_file_path(file_path: &str) -> anyhow::Result<Config> {
    let (server_sdl, source) = ConfigReader::read_file(file_path).await?;
    Config::from_source(source, &server_sdl)
  }
  pub async fn read_file(file_path: &str) -> anyhow::Result<(String, Source)> {
    let mut f = File::open(file_path).await?;
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).await?;
    Ok((String::from_utf8(buffer)?, Source::detect(file_path)?))
  }
  async fn read_over_url(url: reqwest::Url) -> anyhow::Result<(String, Source)> {
    let path = url.path().to_string();
    let resp = reqwest::get(url).await?;
    let source = if let Some(v) = resp.headers().get("content-type") {
      if let Ok(s) = Source::detect(v.to_str()?) {
        s
      } else {
        Source::detect(path.trim_end_matches('/'))?
      }
    } else {
      Source::detect(path.trim_end_matches('/'))?
    };
    let txt = resp.text().await?;
    Ok((txt, source))
  }
  pub fn get_config(&self) -> &Config {
    &self.config
  }
}
