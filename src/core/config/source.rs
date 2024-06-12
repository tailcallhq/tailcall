use thiserror::Error;

use super::Config;
use crate::core::valid::{ValidationError, Validator};

#[derive(Clone)]
pub struct Source {
    pub input_path: String,
    pub input_type: SourceType,
}

#[derive(Clone, Default)]
pub enum SourceType {
    Json,
    Yml,
    #[default]
    GraphQL,
}

const JSON_EXT: &str = "json";
const YML_EXT: &str = "yml";
const GRAPHQL_EXT: &str = "graphql";
const ALL: [SourceType; 3] = [SourceType::Json, SourceType::Yml, SourceType::GraphQL];

#[derive(Debug, Error, PartialEq)]
#[error("Unsupported config extension: {0}")]
pub struct UnsupportedConfigFormat(pub String);

fn normalize_path(path: String) -> String {
    path.replace("\\", "/")
}

impl std::str::FromStr for SourceType {
    type Err = UnsupportedConfigFormat;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(SourceType::Json),
            "yml" | "yaml" => Ok(SourceType::Yml),
            "graphql" | "gql" => Ok(SourceType::GraphQL),
            _ => Err(UnsupportedConfigFormat(s.to_string())),
        }
    }
}

impl SourceType {
    /// Get the file extension for the given format
    pub fn ext(&self) -> &'static str {
        match self {
            SourceType::Json => JSON_EXT,
            SourceType::Yml => YML_EXT,
            SourceType::GraphQL => GRAPHQL_EXT,
        }
    }

    fn ends_with(&self, file: &str) -> bool {
        file.ends_with(&format!(".{}", self.ext()))
    }

    /// Detect the config format from the file name
    pub fn detect(name: &str) -> Result<SourceType, UnsupportedConfigFormat> {
        ALL.into_iter()
            .find(|format| format.ends_with(name))
            .ok_or(UnsupportedConfigFormat(name.to_string()))
    }

    /// Encode the config to the given format
    pub fn encode(&self, config: &Config) -> Result<String, anyhow::Error> {
        match self {
            SourceType::Yml => Ok(config.to_yaml()?),
            SourceType::GraphQL => Ok(config.to_sdl()),
            SourceType::Json => Ok(config.to_json(true)?),
        }
    }
}

impl Source {
    pub fn new(input_path: String, input_type: SourceType) -> Self {
        Self { input_path: normalize_path(input_path), input_type }
    }

    /// Decode the config from the given data
    pub fn decode(&self, data: &str) -> Result<Config, ValidationError<String>> {
        match self.input_type {
            SourceType::Yml => {
                Config::from_yaml(data).map_err(|e| ValidationError::new(e.to_string()))
            }
            SourceType::GraphQL => Config::from_sdl(&self.input_path, data).to_result(),
            SourceType::Json => {
                Config::from_json(data).map_err(|e| ValidationError::new(e.to_string()))
            }
        }
    }
}
