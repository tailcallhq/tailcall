use anyhow::Result;

use crate::config::{Config, Source};
use crate::io::FileIO;

pub struct ConfigWriter<File> {
  file: File,
}

impl<File: FileIO> ConfigWriter<File> {
  pub fn init(file: File) -> Self {
    Self { file }
  }

  pub async fn write(&self, filename: &str, config: &Config) -> Result<()> {
    let source = Source::detect(filename)?;
    let content = source.encode(config)?;
    self.file.write(filename, content.as_bytes()).await?;

    Ok(())
  }
}
