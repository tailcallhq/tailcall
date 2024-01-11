use serde::{Serialize, Deserialize};

use super::is_default;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum OpentelemetryExporter {
  Stdout {
    #[serde(default, skip_serializing_if = "is_default")]
    pretty: bool
  },
  Otlp,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Opentelemetry {
  pub export: Option<OpentelemetryExporter>,
}

impl Opentelemetry {
  pub fn merge_right(&self, other: Self) -> Self {
    other
  }
}