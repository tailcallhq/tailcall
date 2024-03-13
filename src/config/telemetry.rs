use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::KeyValue;
use crate::config::ConfigReaderContext;
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
    pub headers: Vec<KeyValue>,
}

impl OtlpExporter {
    fn merge_right(&self, other: Self) -> Self {
        let mut headers = self.headers.clone();
        headers.extend(other.headers.iter().cloned());

        Self { url: other.url, headers }
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
/// The @telemetry directive facilitates seamless integration with
/// OpenTelemetry, enhancing the observability of your GraphQL services powered
/// by Tailcall.  By leveraging this directive, developers gain access to
/// valuable insights into the performance and behavior of their applications.
pub struct Telemetry {
    pub export: Option<TelemetryExporter>,
    /// The list of headers that will be sent as additional attributes to
    /// telemetry exporters Be careful about **leaking sensitive
    /// information** from requests when enabling the headers that may
    /// contain sensitive data
    #[serde(default, skip_serializing_if = "is_default")]
    pub request_headers: Vec<String>,
}

impl Telemetry {
    pub fn merge_right(mut self, other: Self) -> Self {
        self.export = match (&self.export, other.export) {
            (None, None) => None,
            (None, Some(export)) => Some(export),
            (Some(export), None) => Some(export.clone()),
            (Some(left), Some(right)) => Some(left.merge_right(right.clone())),
        };
        self.request_headers.extend(other.request_headers);

        self
    }

    pub fn render_mustache(&mut self, reader_ctx: &ConfigReaderContext) -> Result<()> {
        if let Some(TelemetryExporter::Otlp(otlp)) = &mut self.export {
            let url_tmpl = Mustache::parse(&otlp.url)?;
            otlp.url = url_tmpl.render(reader_ctx);

            let headers = to_mustache_headers(&otlp.headers).to_result()?;
            otlp.headers = headers
                .into_iter()
                .map(|(key, tmpl)| (key.as_str().to_owned(), tmpl.render(reader_ctx)))
                .map(|(key, value)| KeyValue { key, value })
                .collect();
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_right() {
        let exporter_none = Telemetry { export: None, ..Default::default() };
        let exporter_stdout = Telemetry {
            export: Some(TelemetryExporter::Stdout(StdoutExporter { pretty: true })),
            ..Default::default()
        };
        let exporter_otlp_1 = Telemetry {
            export: Some(TelemetryExporter::Otlp(OtlpExporter {
                url: "test-url".to_owned(),
                headers: vec![KeyValue { key: "header_a".to_owned(), value: "a".to_owned() }],
            })),
            request_headers: vec!["Api-Key-A".to_owned()],
        };
        let exporter_otlp_2 = Telemetry {
            export: Some(TelemetryExporter::Otlp(OtlpExporter {
                url: "test-url-2".to_owned(),
                headers: vec![KeyValue { key: "header_b".to_owned(), value: "b".to_owned() }],
            })),
            request_headers: vec!["Api-Key-B".to_owned()],
        };
        let exporter_prometheus_1 = Telemetry {
            export: Some(TelemetryExporter::Prometheus(PrometheusExporter {
                path: "/metrics".to_owned(),
                format: PrometheusFormat::Text,
            })),
            ..Default::default()
        };
        let exporter_prometheus_2 = Telemetry {
            export: Some(TelemetryExporter::Prometheus(PrometheusExporter {
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
            Telemetry {
                export: Some(TelemetryExporter::Otlp(OtlpExporter {
                    url: "test-url-2".to_owned(),
                    headers: vec![
                        KeyValue { key: "header_a".to_owned(), value: "a".to_owned() },
                        KeyValue { key: "header_b".to_owned(), value: "b".to_owned() }
                    ]
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
