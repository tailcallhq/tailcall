use thiserror::Error;

use crate::config::UnsupportedConfigFormat;

///
/// A list of sources from which a configuration can be created
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum GeneratorSource {
    #[default]
    PROTO,
}

const ALL: &[GeneratorSource] = &[GeneratorSource::PROTO];

const PROTO_EXT: &str = "proto";

#[derive(Debug, Error)]
#[derive(Debug, Error, PartialEq)]
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

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use super::*;

    #[test]
    fn test_from_str() {
        assert_eq!(GeneratorSource::from_str("proto"), Ok(GeneratorSource::PROTO));
        assert!(GeneratorSource::from_str("foo").is_err());
    }

    #[test]
    fn test_ext() {
        assert_eq!(GeneratorSource::PROTO.ext(), "proto");
    }

    #[test]
    fn test_ends_with() {
        let proto = GeneratorSource::PROTO;
        assert!(proto.ends_with("foo.proto"));
        assert!(!proto.ends_with("foo.xml"));
    }

    #[test]
    fn test_detect() {
        assert_eq!(GeneratorSource::detect("foo.proto"), Ok(GeneratorSource::PROTO));
        assert!(GeneratorSource::detect("foo.xml").is_err());
    }
}
