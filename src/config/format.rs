use thiserror::Error;

pub enum Format {
    Json,
    Yml,
    GraphQL,
}

const JSON_EXT: &str = "json";
const YML_EXT: &str = "yml";
const GRAPHQL_EXT: &str = "graphql";
const ALL: [Format; 3] = [Format::Json, Format::Yml, Format::GraphQL];

#[derive(Debug, Error)]
#[error("Unsupported file extension: {0}")]
pub struct UnsupportedFileFormat(String);

impl Format {
    pub fn ext(&self) -> &'static str {
        match self {
            Format::Json => JSON_EXT,
            Format::Yml => YML_EXT,
            Format::GraphQL => GRAPHQL_EXT,
        }
    }

    fn ends_with(&self, file: &str) -> bool {
        file.ends_with(&format!(".{}", self.ext()))
    }

    pub fn detect(name: &str) -> Result<Format, UnsupportedFileFormat> {
        ALL.into_iter()
            .find(|format| format.ends_with(name))
            .ok_or_else(|| UnsupportedFileFormat(name.to_string()))
    }
}
