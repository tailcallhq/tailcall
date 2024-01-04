use anyhow::anyhow;
use url::Url;

use crate::config::{Config, Source};

pub struct ConfigReader {
  pub file_paths: Vec<String>,
}

impl ConfigReader {
  pub fn init<Iter>(file_paths: Iter) -> Self
  where
    Iter: Iterator,
    Iter::Item: AsRef<str>,
  {
    Self { file_paths: file_paths.map(|path| path.as_ref().to_owned()).collect() }
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
  pub async fn from_url(url: Url) -> anyhow::Result<Config> {
    let (st, source) = Self::read_over_url(url).await?;
    Config::from_source(source, &st)
  }
}
