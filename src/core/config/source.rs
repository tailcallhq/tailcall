use std::path::Path;
use std::str::FromStr;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Default, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Source {
    Json,
    Yml,
    #[default]
    GraphQL,
}

impl std::fmt::Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Source::Json => write!(f, "JSON"),
            Source::Yml => write!(f, "YML"),
            Source::GraphQL => write!(f, "GraphQL"),
        }
    }
}

const JSON_EXT: &str = "json";
const YML_EXT: &str = "yml";
const GRAPHQL_EXT: &str = "graphql";

#[derive(Debug, Error, PartialEq)]
pub enum SourceError {
    #[error("Unsupported config extension: {0}")]
    UnsupportedFileFormat(String),
    #[error("Cannot parse")]
    InvalidPath(String),
}

impl std::str::FromStr for Source {
    type Err = SourceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(Source::Json),
            "yml" | "yaml" => Ok(Source::Yml),
            "graphql" | "gql" => Ok(Source::GraphQL),
            _ => Err(SourceError::UnsupportedFileFormat(s.to_string())),
        }
    }
}

impl Source {
    /// Get the file extension for the given format
    pub fn ext(&self) -> &'static str {
        match self {
            Source::Json => JSON_EXT,
            Source::Yml => YML_EXT,
            Source::GraphQL => GRAPHQL_EXT,
        }
    }

    /// Detect the config format from the file name
    pub fn detect(name: &str) -> Result<Source, SourceError> {
        Path::new(name)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(Source::from_str)
            .ok_or(SourceError::InvalidPath(name.to_string()))?
    }
}
