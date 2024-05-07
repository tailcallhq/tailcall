use thiserror::Error;

///
/// A list of sources from which a configuration can be created
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum Source {
    #[default]
    Proto,
}

#[derive(Debug, Error, PartialEq)]
#[error("Unsupported config extension: {0}")]
pub struct UnsupportedFileFormat(String);

impl std::str::FromStr for Source {
    type Err = UnsupportedFileFormat;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "proto" => Ok(Source::Proto),
            _ => Err(UnsupportedFileFormat(s.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use crate::core::config::UnsupportedConfigFormat;

    const ALL: &[Source] = &[Source::Proto];

    const PROTO_EXT: &str = "proto";
    impl Source {
        pub fn ext(&self) -> &'static str {
            match self {
                Source::Proto => PROTO_EXT,
            }
        }

        fn ends_with(&self, content: &str) -> bool {
            content.ends_with(&format!(".{}", self.ext()))
        }

        pub fn detect(name: &str) -> Result<Source, UnsupportedConfigFormat> {
            ALL.iter()
                .find(|format| format.ends_with(name))
                .ok_or(UnsupportedConfigFormat(name.to_string()))
                .cloned()
        }
    }

    #[test]
    fn test_from_str() {
        assert_eq!(Source::from_str("proto"), Ok(Source::Proto));
        assert!(Source::from_str("foo").is_err());
    }

    #[test]
    fn test_ext() {
        assert_eq!(Source::Proto.ext(), "proto");
    }

    #[test]
    fn test_ends_with() {
        let proto = Source::Proto;
        assert!(proto.ends_with("foo.proto"));
        assert!(!proto.ends_with("foo.xml"));
    }

    #[test]
    fn test_detect() {
        assert_eq!(Source::detect("foo.proto"), Ok(Source::Proto));
        assert!(Source::detect("foo.xml").is_err());
    }
}
