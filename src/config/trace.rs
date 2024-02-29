use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::KeyValues;
use crate::helpers::headers::to_mustache_headers;
use crate::is_default;
use crate::mustache::Mustache;
use crate::runtime::TargetRuntimeContext;
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

/// Output the opentelemetry data to otlp collector
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct OtlpExporter {
    pub url: String,
    #[serde(default, skip_serializing_if = "is_default")]
    pub headers: KeyValues,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum TraceExporter {
    Stdout(StdoutExporter),
    Otlp(OtlpExporter),
    Prometheus(PrometheusExporter),
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Trace {
    pub export: Option<TraceExporter>,
}

impl Trace {
    pub fn merge_right(&self, other: Self) -> Self {
        Self { export: other.export.or(self.export.clone()) }
    }

    pub fn render_mustache(&mut self, runtime_ctx: &TargetRuntimeContext) -> Result<()> {
        if let Some(TraceExporter::Otlp(otlp)) = &mut self.export {
            let url_tmpl = Mustache::parse(&otlp.url)?;
            otlp.url = url_tmpl.render(runtime_ctx);

            let headers = to_mustache_headers(&otlp.headers).to_result()?;
            otlp.headers = headers
                .into_iter()
                .map(|(key, tmpl)| (key.as_str().to_owned(), tmpl.render(runtime_ctx)))
                .collect();
        }

        Ok(())
    }
}
