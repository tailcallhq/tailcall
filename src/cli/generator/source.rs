use crate::core::config;
use crate::core::config::SourceError;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ConfigSource {
    Json,
    Yml,
}

impl TryFrom<config::Source> for ConfigSource {
    type Error = SourceError;

    fn try_from(value: config::Source) -> Result<Self, Self::Error> {
        match value {
            config::Source::Json => Ok(Self::Json),
            config::Source::Yml => Ok(Self::Yml),
            config::Source::GraphQL => {
                Err(SourceError::UnsupportedFileFormat(value.ext().to_string()))
            }
        }
    }
}

impl ConfigSource {
    /// Detect the config format from the file name
    pub fn detect(name: &str) -> Result<Self, SourceError> {
        let source = config::Source::detect(name)?;

        ConfigSource::try_from(source)
    }
}
