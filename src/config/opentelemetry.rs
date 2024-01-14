use serde::{Deserialize, Serialize};

use super::{is_default, KeyValues};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StdoutExporter {
  #[serde(default, skip_serializing_if = "is_default")]
  pub pretty: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OtlpExporter {
  pub url: String,
  #[serde(default, skip_serializing_if = "is_default")]
  pub headers: KeyValues,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum OpentelemetryExporter {
  Stdout(StdoutExporter),
  Otlp(OtlpExporter),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OpentelemetryInner {
  pub export: OpentelemetryExporter,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Opentelemetry(pub Option<OpentelemetryInner>);

impl Opentelemetry {
  pub fn merge_right(&self, other: Self) -> Self {
    Self(other.0.or(self.0.clone()))
  }
}
