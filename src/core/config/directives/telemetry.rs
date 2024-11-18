use anyhow::Result;
use serde::{Deserialize, Serialize};
use tailcall_macros::{DirectiveDefinition, InputDefinition};
use tailcall_valid::Validator;

use crate::core::config::{ConfigReaderContext, KeyValue, TelemetryExporterConfig};
use crate::core::helpers::headers::to_mustache_headers;
use crate::core::is_default;
use crate::core::merge_right::MergeRight;
use crate::core::mustache::Mustache;

#[derive(
    Debug,
    Default,
    Clone,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    schemars::JsonSchema,
    DirectiveDefinition,
    InputDefinition,
)]
#[directive_definition(locations = "Schema")]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
/// The @telemetry directive facilitates seamless integration with
/// OpenTelemetry, enhancing the observability of your GraphQL services powered
/// by Tailcall.  By leveraging this directive, developers gain access to
/// valuable insights into the performance and behavior of their applications.
pub struct Telemetry {
    pub export: Option<TelemetryExporterConfig>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::{
        OtlpExporterConfig, PrometheusExporter, PrometheusFormat, StdoutExporter,
    };

    #[test]
    fn merge_right() {
        let exporter_none = Telemetry { export: None, ..Default::default() };
        let exporter_stdout = Telemetry {
            export: Some(TelemetryExporterConfig::Stdout(StdoutExporter {
                pretty: true,
            })),
            ..Default::default()
        };
        let exporter_otlp_1 = Telemetry {
            export: Some(TelemetryExporterConfig::Otlp(OtlpExporterConfig {
                url: "test-url".to_owned(),
                headers: vec![KeyValue { key: "header_a".to_owned(), value: "a".to_owned() }],
            })),
            request_headers: vec!["Api-Key-A".to_owned()],
        };
        let exporter_otlp_2 = Telemetry {
            export: Some(TelemetryExporterConfig::Otlp(OtlpExporterConfig {
                url: "test-url-2".to_owned(),
                headers: vec![KeyValue { key: "header_b".to_owned(), value: "b".to_owned() }],
            })),
            request_headers: vec!["Api-Key-B".to_owned()],
        };
        let exporter_prometheus_1 = Telemetry {
            export: Some(TelemetryExporterConfig::Prometheus(PrometheusExporter {
                path: "/metrics".to_owned(),
                format: PrometheusFormat::Text,
            })),
            ..Default::default()
        };
        let exporter_prometheus_2 = Telemetry {
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
            Telemetry {
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
