use anyhow::Result;

use crate::config::{Config, Source};
use crate::io::file::FileIO;

pub struct ConfigWriter<File> {
  file: File,
}

impl<File: FileIO> ConfigWriter<File> {
  pub fn init(file: File) -> Self {
    Self { file }
  }

  pub async fn write(&self, filename: &String, config: &Config) -> Result<()> {
    let source = Source::detect(filename)?;
    let content = source.encode(config)?;
    self.file.write(filename.as_str(), content.as_bytes()).await?;

    Ok(())
  }
}
