use std::str::FromStr;

use http::header::{HeaderMap, HeaderName, HeaderValue};
use tailcall_valid::{Valid, ValidationError, Validator};
use url::Url;

use crate::core::blueprint::TryFoldConfig;
use crate::core::config::{
    ApolloTelemetryStatic, ConfigModule, KeyValue, PrometheusExporterStatic, StdoutExporterStatic,
    TelemetryExporterConfigStatic,
};
use crate::core::try_fold::TryFold;

#[derive(Debug, Default, Clone)]
pub struct TelemetryRuntime {
    pub export: Option<TelemetryExporterRuntime>,
    pub request_headers: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct OtlpExporterRuntime {
    pub url: Url,
    pub headers: HeaderMap,
}

#[derive(Debug, Clone)]
pub enum TelemetryExporterRuntime {
    Stdout(StdoutExporterStatic),
    Otlp(OtlpExporterRuntime),
    Prometheus(PrometheusExporterStatic),
    Apollo(ApolloTelemetryStatic),
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

pub fn to_opentelemetry<'a>() -> TryFold<'a, ConfigModule, TelemetryRuntime, String> {
    TryFoldConfig::<TelemetryRuntime>::new(|config, up| {
        if let Some(export) = config.telemetry.export.as_ref() {
            let export = match export {
                TelemetryExporterConfigStatic::Stdout(config) => {
                    Valid::succeed(TelemetryExporterRuntime::Stdout(config.clone()))
                }
                TelemetryExporterConfigStatic::Otlp(config) => to_url(&config.url)
                    .zip(to_headers(config.headers.clone()))
                    .map(|(url, headers)| {
                        TelemetryExporterRuntime::Otlp(OtlpExporterRuntime { url, headers })
                    })
                    .trace("otlp"),
                TelemetryExporterConfigStatic::Prometheus(config) => {
                    Valid::succeed(TelemetryExporterRuntime::Prometheus(config.clone()))
                }
                TelemetryExporterConfigStatic::Apollo(apollo) => validate_apollo(apollo.clone())
                    .and_then(|apollo| Valid::succeed(TelemetryExporterRuntime::Apollo(apollo))),
            };

            export.map(|export| TelemetryRuntime {
                export: Some(export),
                request_headers: config.telemetry.request_headers.clone(),
            })
        } else {
            Valid::succeed(up)
        }
    })
}

fn validate_apollo(apollo: ApolloTelemetryStatic) -> Valid<ApolloTelemetryStatic, String> {
    validate_graph_ref(&apollo.graph_ref)
        .map(|_| apollo)
        .trace("apollo.graph_ref")
}

fn validate_graph_ref(graph_ref: &str) -> Valid<(), String> {
    let is_valid = regex::Regex::new(r"^[A-Za-z0-9-_]+@[A-Za-z0-9-_]+$")
        .unwrap()
        .is_match(graph_ref);
    if is_valid {
        Valid::succeed(())
    } else {
        Valid::fail(format!("`graph_ref` should be in the format <graph_id>@<variant> where `graph_id` and `variant` can only contain letters, numbers, '-' and '_'. Found {graph_ref}").to_string())
    }
}

#[cfg(test)]
mod tests {
    use tailcall_valid::Valid;

    use super::validate_graph_ref;

    #[test]
    fn test_validate_graph_ref() {
        let success = || Valid::succeed(());
        let failure = |graph_ref| {
            Valid::fail(format!("`graph_ref` should be in the format <graph_id>@<variant> where `graph_id` and `variant` can only contain letters, numbers, '-' and '_'. Found {graph_ref}").to_string())
        };

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
