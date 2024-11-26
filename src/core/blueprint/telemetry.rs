use std::str::FromStr;

use http::header::{HeaderMap, HeaderName, HeaderValue};
use tailcall_valid::{Valid, Validator};
use url::Url;

use super::{BlueprintError, TryFoldConfig};
use crate::core::config::{
    self, Apollo, ConfigModule, KeyValue, PrometheusExporter, StdoutExporter,
};
use crate::core::directive::DirectiveCodec;
use crate::core::try_fold::TryFold;

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
    Apollo(Apollo),
}

#[derive(Debug, Default, Clone)]
pub struct Telemetry {
    pub export: Option<TelemetryExporter>,
    pub request_headers: Vec<String>,
}

fn to_url(url: &str) -> Valid<Url, BlueprintError> {
    match Url::parse(url).map_err(BlueprintError::UrlParse) {
        Ok(url) => Valid::succeed(url),
        Err(err) => Valid::fail(err),
    }
    .trace("url")
}

fn to_headers(headers: Vec<KeyValue>) -> Valid<HeaderMap, BlueprintError> {
    Valid::from_iter(headers.iter(), |key_value| {
        match HeaderName::from_str(&key_value.key).map_err(BlueprintError::InvalidHeaderName) {
            Ok(name) => Valid::succeed(name),
            Err(err) => Valid::fail(err),
        }
        .zip({
            match HeaderValue::from_str(&key_value.value)
                .map_err(BlueprintError::InvalidHeaderValue)
            {
                Ok(value) => Valid::succeed(value),
                Err(err) => Valid::fail(err),
            }
        })
    })
    .map(HeaderMap::from_iter)
    .trace("headers")
}

pub fn to_opentelemetry<'a>() -> TryFold<'a, ConfigModule, Telemetry, BlueprintError> {
    TryFoldConfig::<Telemetry>::new(|config, up| {
        if let Some(export) = config.telemetry.export.as_ref() {
            let export: Valid<TelemetryExporter, BlueprintError> = match export {
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
                config::TelemetryExporter::Apollo(apollo) => validate_apollo(apollo.clone())
                    .and_then(|apollo| Valid::succeed(TelemetryExporter::Apollo(apollo))),
            };

            export
                .map(|export| Telemetry {
                    export: Some(export),
                    request_headers: config.telemetry.request_headers.clone(),
                })
                .trace(config::Telemetry::trace_name().as_str())
        } else {
            Valid::succeed(up)
        }
    })
}

fn validate_apollo(apollo: Apollo) -> Valid<Apollo, BlueprintError> {
    validate_graph_ref(&apollo.graph_ref)
        .map(|_| apollo)
        .trace("apollo.graph_ref")
}

fn validate_graph_ref(graph_ref: &str) -> Valid<(), BlueprintError> {
    let is_valid = regex::Regex::new(r"^[A-Za-z0-9-_]+@[A-Za-z0-9-_]+$")
        .unwrap()
        .is_match(graph_ref);
    if is_valid {
        Valid::succeed(())
    } else {
        Valid::fail(BlueprintError::InvalidGraphRef(graph_ref.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use tailcall_valid::Valid;

    use super::validate_graph_ref;
    use crate::core::blueprint::BlueprintError;

    #[test]
    fn test_validate_graph_ref() {
        let success = || Valid::succeed(());
        let failure =
            |graph_ref: &str| Valid::fail(BlueprintError::InvalidGraphRef(graph_ref.to_string()));

        assert_eq!(validate_graph_ref("graph_id@variant"), success());
        assert_eq!(
            validate_graph_ref("gr@ph_id@variant"),
            failure("gr@ph_id@variant")
        );
        assert_eq!(validate_graph_ref("graph-Id@variant"), success());
        assert_eq!(
            validate_graph_ref("graph$id@variant1"),
            failure("graph$id@variant1")
        );
        assert_eq!(
            validate_graph_ref("gr@ph_id@variant"),
            failure("gr@ph_id@variant")
        );
    }
}
