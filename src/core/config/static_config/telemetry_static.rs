use anyhow::Result;
use serde::{Deserialize, Serialize};
use tailcall_valid::Validator;

use crate::core::config::{KeyValue, Telemetry};
use crate::core::helpers::headers::to_mustache_headers;
use crate::core::is_default;
use crate::core::macros::MergeRight;
use crate::core::merge_right::MergeRight;
use crate::core::mustache::Mustache;

pub mod apollo_static;
pub mod reader_context;
pub use apollo_static::ApolloTelemetry;
pub use reader_context::ConfigReaderContext;

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
/// The `telemetry` option facilitates seamless integration with OpenTelemetry,
/// enhancing the observability of your GraphQL services powered by Tailcall.
/// By configuring `telemetry`, developers gain access to valuable insights
/// into the performance and behavior of their applications.
pub struct TelemetryStatic {
    // TODO: FIXME
    pub export: Option<TelemetryExporterConfig>,
    /// The list of headers that will be sent as additional attributes to
    /// telemetry exporters Be careful about **leaking sensitive
    /// information** from requests when enabling the headers that may
    /// contain sensitive data
    #[serde(default, skip_serializing_if = "is_default")]
    pub request_headers: Vec<String>,
}

impl TelemetryStatic {
    pub fn merge_right(mut self, other: Self) -> Self {
        self.export = match (&self.export, other.export) {
            (None, None) => None,
            (None, Some(export)) => Some(export),
            (Some(export), None) => Some(export.clone()),
            (Some(left), Some(right)) => Some(left.clone().merge_right(right.clone())),
        };
        self.request_headers.extend(other.request_headers);

        self
    }

    pub fn render_mustache(&mut self, reader_ctx: &ConfigReaderContext) -> Result<()> {
        match &mut self.export {
            Some(TelemetryExporterConfig::Otlp(otlp)) => {
                let url_tmpl = Mustache::parse(&otlp.url);
                otlp.url = url_tmpl.render(reader_ctx);

                let headers = to_mustache_headers(&otlp.headers).to_result()?;
                otlp.headers = headers
                    .into_iter()
                    .map(|(key, tmpl)| (key.as_str().to_owned(), tmpl.render(reader_ctx)))
                    .map(|(key, value)| KeyValue { key, value })
                    .collect();
            }
            Some(TelemetryExporterConfig::Apollo(apollo)) => apollo.render_mustache(reader_ctx)?,
            _ => {}
        }

        Ok(())
    }
}

impl From<Telemetry> for TelemetryStatic {
    fn from(telemetry: Telemetry) -> Self {
        Self {
            export: telemetry.export,
            request_headers: telemetry.request_headers,
        }
    }
}

mod defaults {
    pub mod prometheus {
        pub fn path() -> String {
            "/metrics".to_owned()
        }
    }
}

/// Output the opentelemetry data to the stdout. Mostly used for debug purposes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, schemars::JsonSchema, MergeRight)]
pub struct StdoutExporter {
    /// Output to stdout in pretty human-readable format
    #[serde(default, skip_serializing_if = "is_default")]
    pub pretty: bool,
}

/// Output the opentelemetry data to otlp collector
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, schemars::JsonSchema, MergeRight)]
pub struct OtlpExporterConfig {
    pub url: String,
    #[serde(default, skip_serializing_if = "is_default")]
    pub headers: Vec<KeyValue>,
}

/// Output format for prometheus data
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
pub enum PrometheusFormat {
    #[default]
    Text,
    Protobuf,
}

/// Output the telemetry metrics data to prometheus server
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
pub struct PrometheusExporter {
    #[serde(
        default = "defaults::prometheus::path",
        skip_serializing_if = "is_default"
    )]
    pub path: String,
    #[serde(default, skip_serializing_if = "is_default")]
    pub format: PrometheusFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, schemars::JsonSchema, MergeRight)]
pub enum TelemetryExporterConfig {
    Stdout(StdoutExporter),
    Otlp(OtlpExporterConfig),
    Prometheus(PrometheusExporter),
    Apollo(ApolloTelemetry),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_right() {
        let exporter_none = TelemetryStatic { export: None, ..Default::default() };
        let exporter_stdout = TelemetryStatic {
            export: Some(TelemetryExporterConfig::Stdout(StdoutExporter {
                pretty: true,
            })),
            ..Default::default()
        };
        let exporter_otlp_1 = TelemetryStatic {
            export: Some(TelemetryExporterConfig::Otlp(OtlpExporterConfig {
                url: "test-url".to_owned(),
                headers: vec![KeyValue { key: "header_a".to_owned(), value: "a".to_owned() }],
            })),
            request_headers: vec!["Api-Key-A".to_owned()],
        };
        let exporter_otlp_2 = TelemetryStatic {
            export: Some(TelemetryExporterConfig::Otlp(OtlpExporterConfig {
                url: "test-url-2".to_owned(),
                headers: vec![KeyValue { key: "header_b".to_owned(), value: "b".to_owned() }],
            })),
            request_headers: vec!["Api-Key-B".to_owned()],
        };
        let exporter_prometheus_1 = TelemetryStatic {
            export: Some(TelemetryExporterConfig::Prometheus(PrometheusExporter {
                path: "/metrics".to_owned(),
                format: PrometheusFormat::Text,
            })),
            ..Default::default()
        };
        let exporter_prometheus_2 = TelemetryStatic {
            export: Some(TelemetryExporterConfig::Prometheus(PrometheusExporter {
                path: "/prom".to_owned(),
                format: PrometheusFormat::Protobuf,
            })),
            ..Default::default()
        };

        assert_eq!(
            exporter_none.clone().merge_right(exporter_none.clone()),
            exporter_none
        );

        assert_eq!(
            exporter_stdout.clone().merge_right(exporter_none.clone()),
            exporter_stdout
        );

        assert_eq!(
            exporter_none.clone().merge_right(exporter_otlp_1.clone()),
            exporter_otlp_1
        );

        assert_eq!(
            exporter_stdout.clone().merge_right(exporter_otlp_1.clone()),
            exporter_otlp_1
        );

        assert_eq!(
            exporter_stdout.clone().merge_right(exporter_stdout.clone()),
            exporter_stdout
        );

        assert_eq!(
            exporter_otlp_1.clone().merge_right(exporter_otlp_2.clone()),
            TelemetryStatic {
                export: Some(TelemetryExporterConfig::Otlp(OtlpExporterConfig {
                    url: "test-url-2".to_owned(),
                    headers: vec![KeyValue { key: "header_b".to_owned(), value: "b".to_owned() }]
                })),
                request_headers: vec!["Api-Key-A".to_string(), "Api-Key-B".to_string(),]
            }
        );

        assert_eq!(
            exporter_prometheus_1
                .clone()
                .merge_right(exporter_prometheus_2.clone()),
            exporter_prometheus_2
        );
    }
}
