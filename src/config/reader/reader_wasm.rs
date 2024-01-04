use anyhow::anyhow;
use url::Url;

use crate::config::reader::ConfigReader;
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
}
