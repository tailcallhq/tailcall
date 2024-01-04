use anyhow::Result;
#[cfg(feature = "default")]
use tokio::{fs::File, io::AsyncWriteExt};

use crate::config::{Config, Source};

pub struct ConfigWriter {
  config: Config,
}

impl ConfigWriter {
  pub fn init(config: Config) -> Self {
    Self { config }
  }

  pub async fn write(&self, filename: &String) -> Result<()> {
    let _contents = match Source::detect(filename)? {
      Source::GraphQL => self.config.to_sdl(),
      Source::Json => self.config.to_json(true)?,
      Source::Yml => self.config.to_yaml()?,
    };
    #[cfg(feature = "default")]
    let mut file = File::create(filename).await?;
    #[cfg(feature = "default")]
    file.write_all(_contents.as_bytes()).await?;

    Ok(())
  }
}
