use hyper::HeaderMap;
use url::Url;

use crate::config::StdoutExporter;

#[derive(Debug, Clone)]
pub struct OtlpExporter {
  pub url: Url,
  pub headers: HeaderMap,
}

#[derive(Debug, Clone)]
pub enum OpentelemetryExporter {
  Stdout(StdoutExporter),
  Otlp(OtlpExporter),
}

#[derive(Debug, Clone)]
pub struct OpentelemetryInner {
  pub export: OpentelemetryExporter,
}

#[derive(Debug, Default, Clone)]
pub struct Opentelemetry(pub Option<OpentelemetryInner>);
