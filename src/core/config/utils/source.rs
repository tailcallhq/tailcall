use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tailcall_valid::ValidationError;
use thiserror::Error;

use crate::core::config::Config;

/// The `Source` is responsible for reading a config file from disk.
#[derive(Clone, Default, Debug, Serialize, Deserialize, JsonSchema)]
pub enum SourceUtil {
    Json,
    #[default]
    Yml
}

impl std::fmt::Display for SourceUtil {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SourceUtil::Json => write!(f, "JSON"),
            SourceUtil::Yml => write!(f, "YML"),
        }
    }
}

const JSON_EXT: &str = "json";
const YML_EXT: &str = "yml";
const ALL: [SourceUtil; 2] = [SourceUtil::Json, SourceUtil::Yml];

#[derive(Debug, Error, PartialEq)]
#[error("Unsupported config extension: {0}")]
pub struct UnsupportedConfigFormat(pub String);

impl std::str::FromStr for SourceUtil {
    type Err = UnsupportedConfigFormat;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(SourceUtil::Json),
            "yml" | "yaml" => Ok(SourceUtil::Yml),
            _ => Err(UnsupportedConfigFormat(s.to_string())),
        }
    }
}

impl SourceUtil {
    /// Get the file extension for the given format
    pub fn ext(&self) -> &'static str {
        match self {
            SourceUtil::Json => JSON_EXT,
            SourceUtil::Yml => YML_EXT,
        }
    }

    fn ends_with(&self, file: &str) -> bool {
        file.ends_with(&format!(".{}", self.ext()))
    }

    /// Detect the config format from the file name
    pub fn detect(name: &str) -> Result<SourceUtil, UnsupportedConfigFormat> {
        ALL.into_iter()
            .find(|format| format.ends_with(name))
            .ok_or(UnsupportedConfigFormat(name.to_string()))
    }

    /// Encode the config to the given format
    pub fn encode(&self, config: &Config) -> Result<String, anyhow::Error> {
        match self {
            SourceUtil::Yml => Ok(config.to_yaml()?),
            SourceUtil::Json => Ok(config.to_json(true)?),
        }
    }

    /// Decode the config from the given data
    pub fn decode(&self, data: &str) -> Result<Config, ValidationError<String>> {
        match self {
            SourceUtil::Yml => Config::from_yaml(data).map_err(|e| ValidationError::new(e.to_string())),
            SourceUtil::Json => {
                Config::from_json(data).map_err(|e| ValidationError::new(e.to_string()))
            }
        }
    }
}
