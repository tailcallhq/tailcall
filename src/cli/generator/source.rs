use thiserror::Error;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ConfigSource {
    Json,
    Yml,
}

impl ConfigSource {
    fn ext(&self) -> &str {
        match self {
            Self::Json => "json",
            Self::Yml => "yml",
        }
    }

    fn ends_with(&self, file: &str) -> bool {
        file.ends_with(&format!(".{}", self.ext()))
    }

    /// Detect the config format from the file name
    pub fn detect(name: &str) -> Result<Self, UnsupportedFileFormat> {
        const ALL: &[ConfigSource] = &[ConfigSource::Json, ConfigSource::Yml];

        ALL.iter()
            .find(|format| format.ends_with(name))
            .copied()
            .ok_or(UnsupportedFileFormat(name.to_string()))
    }
}

#[derive(Debug, Error, PartialEq)]
#[error("Unsupported config extension: {0}")]
pub struct UnsupportedFileFormat(String);
