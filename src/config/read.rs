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
  pub async fn serialize_config(&mut self, path: &str) -> anyhow::Result<()> {
    let conf = if let Ok(url) = reqwest::Url::parse(path) {
      let (st, source) = Self::read_over_url(url).await?;
      Config::from_source(source, &st)?
    } else {
      let path = path.trim_end_matches('/');
      Config::from_file_path(path).await?
    };
    self.config = self.config.clone().merge_right(&conf);
    Ok(())
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
