use thiserror::Error;

use super::Config;

#[derive(Clone)]
pub enum Source {
    Json,
    Yml,
    GraphQL,
}

const JSON_EXT: &str = "json";
const YML_EXT: &str = "yml";
const GRAPHQL_EXT: &str = "graphql";
const ALL: [Source; 3] = [Source::Json, Source::Yml, Source::GraphQL];

#[derive(Debug, Error)]
#[error("Unsupported file extension: {0}")]
pub struct UnsupportedFileFormat(String);

impl std::str::FromStr for Source {
    type Err = UnsupportedFileFormat;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(Source::Json),
            "yml" | "yaml" => Ok(Source::Yml),
            "graphql" | "gql" => Ok(Source::GraphQL),
            _ => Err(UnsupportedFileFormat(s.to_string())),
        }
    }
}

impl Source {
    pub fn ext(&self) -> &'static str {
        match self {
            Source::Json => JSON_EXT,
            Source::Yml => YML_EXT,
            Source::GraphQL => GRAPHQL_EXT,
        }
    }

    fn ends_with(&self, file: &str) -> bool {
        file.ends_with(&format!(".{}", self.ext()))
    }

    pub fn detect(name: &str) -> Result<Source, UnsupportedFileFormat> {
        ALL.into_iter()
            .find(|format| format.ends_with(name))
            .ok_or(UnsupportedFileFormat(name.to_string()))
    }

    pub fn encode(&self, config: &Config) -> Result<String, anyhow::Error> {
        match self {
            Source::Yml => Ok(config.to_yaml()?),
            Source::GraphQL => Ok(config.to_sdl()),
            Source::Json => Ok(config.to_json(true)?),
        }
    }
}
