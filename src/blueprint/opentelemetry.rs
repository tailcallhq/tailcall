use hyper::header::HeaderValue;
use hyper::HeaderMap;
use url::Url;

use super::init_context::InitContext;
use super::TryFoldConfig;
use crate::directive::DirectiveCodec;
use crate::helpers::headers::to_mustache_headers;
use crate::mustache::Mustache;
use crate::try_fold::TryFold;
use crate::valid::{Valid, ValidationError};
use crate::{
    config::{self, ConfigModule, KeyValues},
    valid::Validator,
};

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

fn to_url(url: &str, init_context: &InitContext) -> Valid<Url, String> {
    Valid::from(Mustache::parse(url).map_err(|e| ValidationError::new(e.to_string())))
        .and_then(|url| {
            Valid::from(
                Url::parse(&url.render(init_context))
                    .map_err(|e| ValidationError::new(e.to_string())),
            )
        })
        .trace("url")
}

fn to_headers(headers: &KeyValues, init_context: &InitContext) -> Valid<HeaderMap, String> {
    to_mustache_headers(headers)
        .and_then(|headers| {
            Valid::from_iter(headers, |(k, v)| {
                Valid::from(
                    HeaderValue::from_str(&v.render(init_context))
                        .map_err(|e| ValidationError::new(e.to_string())),
                )
                .map(|v| (k, v))
            })
        })
        .map(HeaderMap::from_iter)
        .trace("headers")
}

pub fn to_opentelemetry<'a>() -> TryFold<'a, ConfigModule, Opentelemetry, String> {
    TryFoldConfig::<Opentelemetry>::new(|config, up| {
        if let Some(opentelemetry) = config.opentelemetry.0.as_ref() {
            let init_context = InitContext::from(&config.server);

            let export = match &opentelemetry.export {
                config::OpentelemetryExporter::Stdout(config) => {
                    Valid::succeed(OpentelemetryExporter::Stdout(config.clone()))
                }
                config::OpentelemetryExporter::Otlp(config) => to_url(&config.url, &init_context)
                    .zip(to_headers(&config.headers, &init_context))
                    .map(|(url, headers)| {
                        OpentelemetryExporter::Otlp(OtlpExporter { url, headers })
                    })
                    .trace("otlp"),
            };

            export
                .map(|export| Opentelemetry(Some(OpentelemetryInner { export })))
                .trace(config::Opentelemetry::trace_name().as_str())
        } else {
            Valid::succeed(up)
        }
    })
}
