use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::KeyValues;
use crate::config::{Apollo, ConfigReaderContext};
use crate::helpers::headers::to_mustache_headers;
use crate::is_default;
use crate::mustache::Mustache;
use crate::valid::Validator;

mod defaults {
    pub mod prometheus {
        pub fn path() -> String {
            "/metrics".to_owned()
        }
    }
}

/// Output the opentelemetry data to the stdout. Mostly used for debug purposes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct StdoutExporter {
    /// Output to stdout in pretty human-readable format
    #[serde(default, skip_serializing_if = "is_default")]
    pub pretty: bool,
}

impl StdoutExporter {
    fn merge_right(&self, other: Self) -> Self {
        Self { pretty: other.pretty || self.pretty }
    }
}

/// Output the opentelemetry data to otlp collector
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct OtlpExporter {
    pub url: String,
    #[serde(default, skip_serializing_if = "is_default")]
    pub headers: KeyValues,
}

impl OtlpExporter {
    fn merge_right(&self, other: Self) -> Self {
        let mut headers = other.headers.0;
        headers.extend(self.headers.iter().map(|(k, v)| (k.clone(), v.clone())));

        Self { url: other.url, headers: KeyValues(headers) }
    }
}

/// Output format for prometheus data
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum PrometheusFormat {
    #[default]
    Text,
    Protobuf,
}

/// Output the telemetry metrics data to prometheus server
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PrometheusExporter {
    #[serde(
        default = "defaults::prometheus::path",
        skip_serializing_if = "is_default"
    )]
    pub path: String,
    #[serde(default, skip_serializing_if = "is_default")]
    pub format: PrometheusFormat,
}

impl PrometheusExporter {
    fn merge_right(&self, other: Self) -> Self {
        other
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum TelemetryExporter {
    Stdout(StdoutExporter),
    Otlp(OtlpExporter),
    Prometheus(PrometheusExporter),
    Apollo(Apollo),
}

impl TelemetryExporter {
    fn merge_right(&self, other: Self) -> Self {
        match (self, other) {
            (TelemetryExporter::Stdout(left), TelemetryExporter::Stdout(right)) => {
                TelemetryExporter::Stdout(left.merge_right(right))
            }
            (TelemetryExporter::Otlp(left), TelemetryExporter::Otlp(right)) => {
                TelemetryExporter::Otlp(left.merge_right(right))
            }
            (TelemetryExporter::Prometheus(left), TelemetryExporter::Prometheus(right)) => {
                TelemetryExporter::Prometheus(left.merge_right(right))
            }
            (_, other) => other,
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Telemetry {
    pub export: Option<TelemetryExporter>,
}

impl Telemetry {
    pub fn merge_right(&self, other: Self) -> Self {
        let export = match (&self.export, other.export) {
            (None, None) => None,
            (None, Some(export)) => Some(export),
            (Some(export), None) => Some(export.clone()),
            (Some(left), Some(right)) => Some(left.merge_right(right.clone())),
        };

        Self { export }
    }

    pub fn render_mustache(&mut self, reader_ctx: &ConfigReaderContext) -> Result<()> {
        match &mut self.export {
            Some(TelemetryExporter::Otlp(otlp)) => {
                let url_tmpl = Mustache::parse(&otlp.url)?;
                otlp.url = url_tmpl.render(reader_ctx);

                let headers = to_mustache_headers(&otlp.headers).to_result()?;
                otlp.headers = headers
                    .into_iter()
                    .map(|(key, tmpl)| (key.as_str().to_owned(), tmpl.render(reader_ctx)))
                    .collect();
            }
            Some(TelemetryExporter::Apollo(apollo)) => apollo.render_mustache(reader_ctx)?,
            _ => {}
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_right() {
        let exporter_none = Telemetry { export: None };
        let exporter_stdout = Telemetry {
            export: Some(TelemetryExporter::Stdout(StdoutExporter { pretty: true })),
        };
        let exporter_otlp_1 = Telemetry {
            export: Some(TelemetryExporter::Otlp(OtlpExporter {
                url: "test-url".to_owned(),
                headers: KeyValues::from_iter([("header_a".to_owned(), "a".to_owned())]),
            })),
        };
        let exporter_otlp_2 = Telemetry {
            export: Some(TelemetryExporter::Otlp(OtlpExporter {
                url: "test-url-2".to_owned(),
                headers: KeyValues::from_iter([("header_b".to_owned(), "b".to_owned())]),
            })),
        };
        let exporter_prometheus_1 = Telemetry {
            export: Some(TelemetryExporter::Prometheus(PrometheusExporter {
                path: "/metrics".to_owned(),
                format: PrometheusFormat::Text,
            })),
        };
        let exporter_prometheus_2 = Telemetry {
            export: Some(TelemetryExporter::Prometheus(PrometheusExporter {
                path: "/prom".to_owned(),
                format: PrometheusFormat::Protobuf,
            })),
        };

        assert_eq!(
            exporter_none.merge_right(exporter_none.clone()),
            exporter_none
        );

        assert_eq!(
            exporter_stdout.merge_right(exporter_none.clone()),
            exporter_stdout
        );

        assert_eq!(
            exporter_none.merge_right(exporter_otlp_1.clone()),
            exporter_otlp_1
        );

        assert_eq!(
            exporter_stdout.merge_right(exporter_otlp_1.clone()),
            exporter_otlp_1
        );

        assert_eq!(
            exporter_stdout.merge_right(exporter_stdout.clone()),
            exporter_stdout
        );

        assert_eq!(
            exporter_otlp_1.merge_right(exporter_otlp_2.clone()),
            Telemetry {
                export: Some(TelemetryExporter::Otlp(OtlpExporter {
                    url: "test-url-2".to_owned(),
                    headers: KeyValues::from_iter([
                        ("header_a".to_owned(), "a".to_owned()),
                        ("header_b".to_owned(), "b".to_owned())
                    ]),
                })),
            }
        );

        assert_eq!(
            exporter_prometheus_1.merge_right(exporter_prometheus_2.clone()),
            exporter_prometheus_2
        );
    }
}
