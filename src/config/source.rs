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
    let re = regex::Regex::new(r"\.(graphql|yml|yaml|json)$").unwrap();
    if let Some(mat) = re.captures(name) {
      ALL
          .into_iter()
          .find(|format| format.ends_with(&mat[0]))
          .ok_or(UnsupportedFileFormat(name.to_string()))
    } else {
      Err(UnsupportedFileFormat(name.to_string()))
    }
  }
}