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
    let source = Source::detect(filename)?;

    let _contents = source.encode(self.config.clone())?;
    #[cfg(feature = "default")]
    write_file(filename, _contents.as_bytes()).await?;

    Ok(())
  }
}

#[cfg(feature = "default")]
async fn write_file(filename: &String, contents: &[u8]) -> Result<()> {
  let mut file = File::create(filename).await?;
  file.write_all(contents).await?;
  Ok(())
}
