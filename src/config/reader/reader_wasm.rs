use crate::config::reader::ConfigReader;
use anyhow::anyhow;
use url::Url;

use crate::config::{Config, Source};

impl ConfigReader {
  pub async fn read(&self) -> anyhow::Result<Config> {
    let mut config = Config::default();
    for path in &self.file_paths {
      let url = Url::parse(path)?;
      let conf = Self::from_url(url).await?;
      config = config.clone().merge_right(&conf);
    }
    Ok(config)
  }
  async fn read_over_url(url: Url) -> anyhow::Result<(String, Source)> {
    let path = url.path().to_string();
    let resp = reqwest::get(url).await?;
    if !resp.status().is_success() {
      return Err(anyhow!("Read over URL failed with status code: {}", resp.status()));
    }
    let source = if let Some(v) = resp.headers().get("content-type") {
      if let Ok(s) = Source::detect_content_type(v.to_str()?) {
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
  async fn from_url(url: Url) -> anyhow::Result<Config> {
    let (st, source) = Self::read_over_url(url).await?;
    Config::from_source(source, &st)
  }
}
