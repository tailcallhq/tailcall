use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::KeyValues;
use crate::helpers::headers::to_mustache_headers;
use crate::is_default;
use crate::mustache::Mustache;
use crate::runtime::TargetRuntimeContext;
use crate::valid::Validator;

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum TraceExporter {
    Stdout(StdoutExporter),
    Otlp(OtlpExporter),
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
