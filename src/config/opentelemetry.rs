use serde::{Serialize, Deserialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub enum OpenTelemetryExporter {
  #[default]
  None,
  Stdout,
  OTLP,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct OpenTelemetry {
  pub export: OpenTelemetryExporter,
}