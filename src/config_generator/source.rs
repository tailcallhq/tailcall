use thiserror::Error;

use crate::config::UnsupportedConfigFormat;

///
/// A list of sources from which a configuration can be created
///
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum GeneratorSource {
    PROTO,
}

const ALL: &[GeneratorSource] = &[GeneratorSource::PROTO];

const PROTO_EXT: &str = "proto";

#[derive(Debug, Error)]
#[error("Unsupported config extension: {0}")]
pub struct UnsupportedFileFormat(String);

impl std::str::FromStr for GeneratorSource {
    type Err = UnsupportedFileFormat;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "proto" => Ok(GeneratorSource::PROTO),
            _ => Err(UnsupportedFileFormat(s.to_string())),
        }
    }
}

impl GeneratorSource {
    pub fn ext(&self) -> &'static str {
        match self {
            GeneratorSource::PROTO => PROTO_EXT,
        }
    }

    fn ends_with(&self, content: &str) -> bool {
        content.ends_with(&format!(".{}", self.ext()))
    }

    pub fn detect(name: &str) -> Result<GeneratorSource, UnsupportedConfigFormat> {
        ALL.iter()
            .find(|format| format.ends_with(name))
            .ok_or(UnsupportedConfigFormat(name.to_string()))
            .cloned()
    }
}
