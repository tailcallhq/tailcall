use std::str::FromStr;

use hyper::header::{HeaderName, HeaderValue};
use hyper::HeaderMap;
use url::Url;

use super::TryFoldConfig;
use crate::config::{self, ConfigModule, KeyValue, PrometheusExporter, StdoutExporter};
use crate::directive::DirectiveCodec;
use crate::try_fold::TryFold;
use crate::valid::{Valid, ValidationError, Validator};

#[derive(Debug, Clone)]
pub struct OtlpExporter {
    pub url: Url,
    pub headers: HeaderMap,
}

#[derive(Debug, Clone)]
pub enum TelemetryExporter {
    Stdout(StdoutExporter),
    Otlp(OtlpExporter),
    Prometheus(PrometheusExporter),
}

#[derive(Debug, Default, Clone)]
pub struct Telemetry {
    pub export: Option<TelemetryExporter>,
}

fn to_url(url: &str) -> Valid<Url, String> {
    Valid::from(Url::parse(url).map_err(|e| ValidationError::new(e.to_string()))).trace("url")
}

fn to_headers(headers: Vec<KeyValue>) -> Valid<HeaderMap, String> {
    Valid::from_iter(headers.iter(), |key_value| {
        Valid::from(
            HeaderName::from_str(&key_value.key)
                .map_err(|err| ValidationError::new(err.to_string())),
        )
        .zip(Valid::from(
            HeaderValue::from_str(&key_value.value)
                .map_err(|err| ValidationError::new(err.to_string())),
        ))
    })
    .map(HeaderMap::from_iter)
    .trace("headers")
}

pub fn to_opentelemetry<'a>() -> TryFold<'a, ConfigModule, Telemetry, String> {
    TryFoldConfig::<Telemetry>::new(|config, up| {
        if let Some(export) = config.opentelemetry.export.as_ref() {
            let export = match export {
                config::TelemetryExporter::Stdout(config) => {
                    Valid::succeed(TelemetryExporter::Stdout(config.clone()))
                }
                config::TelemetryExporter::Otlp(config) => to_url(&config.url)
                    .zip(to_headers(config.headers.clone()))
                    .map(|(url, headers)| TelemetryExporter::Otlp(OtlpExporter { url, headers }))
                    .trace("otlp"),
                config::TelemetryExporter::Prometheus(config) => {
                    Valid::succeed(TelemetryExporter::Prometheus(config.clone()))
                }
            };

            export
                .map(|export| Telemetry { export: Some(export) })
                .trace(config::Telemetry::trace_name().as_str())
        } else {
            Valid::succeed(up)
        }
    })
}
