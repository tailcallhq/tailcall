use anyhow::anyhow;
use thiserror::Error;

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
    ALL
      .into_iter()
      .find(|format| format.ends_with(name))
      .ok_or_else(|| UnsupportedFileFormat(name.to_string()))
  }
  pub fn try_parse_and_detect(val: &str) -> anyhow::Result<Self> {
    // needs improvement
    match serde_json::from_str::<serde_json::Value>(val) {
      Ok(_) => Ok(Self::Json),
      Err(_) => match serde_yaml::from_str::<serde_yaml::Value>(val) {
        Ok(_) => Ok(Self::Yml),
        Err(_) => match async_graphql::parser::parse_schema(val) {
          Ok(_) => Ok(Self::GraphQL),
          Err(e) => Err(anyhow!(e)),
        },
      },
    }
  }
}
