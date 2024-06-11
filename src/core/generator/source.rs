use thiserror::Error;
use url::Url;

///
/// A list of sources from which a configuration can be created
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum ImportSource {
    #[default]
    Proto,
    Url,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ConfigSource {
    Json,
    Yml,
}

impl ImportSource {
    fn ext(&self) -> Option<&str> {
        match self {
            ImportSource::Proto => Some("proto"),
            ImportSource::Url => None,
        }
    }

    fn ends_with(&self, src: &str) -> bool {
        if let Some(ext) = self.ext() {
            return src.ends_with(&format!(".{}", ext));
        }
        false
    }

    fn is_url(self, src: &str) -> bool {
        Url::parse(src).is_ok()
    }

    /// Detect the config format from the src
    pub fn detect(name: &str) -> Result<Self, UnsupportedFileFormat> {
        const ALL: &[ImportSource] = &[ImportSource::Proto, ImportSource::Url];

        ALL.iter()
            .find(|format| match format {
                ImportSource::Proto => format.ends_with(name),
                ImportSource::Url => format.is_url(name),
            })
            .copied()
            .ok_or(UnsupportedFileFormat(name.to_string()))
    }
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

impl std::str::FromStr for ImportSource {
    type Err = UnsupportedFileFormat;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "proto" => Ok(ImportSource::Proto),
            "url" => Ok(ImportSource::Url),
            _ => Err(UnsupportedFileFormat(s.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_detect_proto_import_source() {
        assert_eq!(
            ImportSource::detect("./news.proto"),
            Ok(ImportSource::Proto)
        );
        assert!(ImportSource::detect("./jsonplaceholder.txt").is_err());
    }

    #[test]
    fn test_detect_url_import_source() {
        assert_eq!(
            ImportSource::detect("http://www.google.com"),
            Ok(ImportSource::Url)
        );
        assert_eq!(
            ImportSource::detect("https://www.google.com"),
            Ok(ImportSource::Url)
        );
        assert_eq!(
            ImportSource::detect("https://google.com"),
            Ok(ImportSource::Url)
        );
    }

    #[test]
    fn test_from_str() {
        assert_eq!(ImportSource::from_str("proto"), Ok(ImportSource::Proto));
        assert_eq!(ImportSource::from_str("PROTO"), Ok(ImportSource::Proto));

        assert_eq!(ImportSource::from_str("url"), Ok(ImportSource::Url));
        assert_eq!(ImportSource::from_str("URL"), Ok(ImportSource::Url));

        assert!(ImportSource::from_str("foo").is_err());
    }
}
