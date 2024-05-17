use thiserror::Error;

///
/// A list of sources from which a configuration can be created
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum Source {
    #[default]
    Proto,
    Url,
}

#[derive(Debug, Error, PartialEq)]
#[error("Unsupported config extension: {0}")]
pub struct UnsupportedFileFormat(String);

impl std::str::FromStr for Source {
    type Err = UnsupportedFileFormat;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "proto" => Ok(Source::Proto),
            "url" => Ok(Source::Url),
            _ => Err(UnsupportedFileFormat(s.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_from_str() {
        assert_eq!(Source::from_str("proto"), Ok(Source::Proto));
        assert_eq!(Source::from_str("PROTO"), Ok(Source::Proto));

        assert_eq!(Source::from_str("url"), Ok(Source::Url));
        assert_eq!(Source::from_str("URL"), Ok(Source::Url));

        assert!(Source::from_str("foo").is_err());
    }
}
