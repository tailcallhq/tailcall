use anyhow::Result;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::config::{Config, Source};

pub struct ConfigWriter {
  config: Config,
}

impl ConfigWriter {
  pub fn init(config: Config) -> Self {
    Self { config }
  }

  pub async fn write(&self, filename: &String) -> Result<()> {
    let contents = match Source::detect(filename)? {
      Source::GraphQL => self.config.to_sdl(),
      Source::Json => self.config.to_json(true)?,
      Source::Yml => self.config.to_yaml()?,
    };

    let mut file = File::create(filename).await?;
    file.write_all(contents.as_bytes()).await?;

    Ok(())
  }
}
