use std::str::FromStr;

use hyper::header::{HeaderName, HeaderValue};
use hyper::HeaderMap;
use url::Url;

use super::TryFoldConfig;
use crate::config::{self, ConfigModule, KeyValues, PrometheusExporter, StdoutExporter};
use crate::directive::DirectiveCodec;
use crate::try_fold::TryFold;
use crate::valid::{Valid, ValidationError, Validator};

#[derive(Debug, Clone)]
pub struct OtlpExporter {
    pub url: Url,
    pub headers: HeaderMap,
}

#[derive(Debug, Clone)]
pub enum OpentelemetryExporter {
    Stdout(StdoutExporter),
    Otlp(OtlpExporter),
    Prometheus(PrometheusExporter),
}

#[derive(Debug, Clone)]
pub struct OpentelemetryInner {
    pub export: OpentelemetryExporter,
}

#[derive(Debug, Default, Clone)]
pub struct Opentelemetry(pub Option<OpentelemetryInner>);

fn to_url(url: &str) -> Valid<Url, String> {
    Valid::from(Url::parse(url).map_err(|e| ValidationError::new(e.to_string()))).trace("url")
}

fn to_headers(headers: &KeyValues) -> Valid<HeaderMap, String> {
    Valid::from_iter(headers.iter(), |(k, v)| {
        Valid::from(HeaderName::from_str(k).map_err(|err| ValidationError::new(err.to_string())))
            .zip(Valid::from(
                HeaderValue::from_str(v).map_err(|err| ValidationError::new(err.to_string())),
            ))
    })
    .map(HeaderMap::from_iter)
    .trace("headers")
}

pub fn to_opentelemetry<'a>() -> TryFold<'a, ConfigModule, Opentelemetry, String> {
    TryFoldConfig::<Opentelemetry>::new(|config, up| {
        if let Some(export) = config.opentelemetry.export.as_ref() {
            let export = match export {
                config::TraceExporter::Stdout(config) => {
                    Valid::succeed(OpentelemetryExporter::Stdout(config.clone()))
                }
                config::TraceExporter::Otlp(config) => to_url(&config.url)
                    .zip(to_headers(&config.headers))
                    .map(|(url, headers)| {
                        OpentelemetryExporter::Otlp(OtlpExporter { url, headers })
                    })
                    .trace("otlp"),
                config::TraceExporter::Prometheus(config) => {
                    Valid::succeed(OpentelemetryExporter::Prometheus(config.clone()))
                }
            };

            export
                .map(|export| Opentelemetry(Some(OpentelemetryInner { export })))
                .trace(config::Trace::trace_name().as_str())
        } else {
            Valid::succeed(up)
        }
    })
}
