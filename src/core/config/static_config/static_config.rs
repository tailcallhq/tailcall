use derive_getters::Getters;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tailcall_macros::MergeRight;
use tailcall_valid::{Valid, Validator};

use super::{LinkStatic, ServerStatic, TelemetryStatic, UpstreamStatic};
use crate::core::config::{Config, Source};
use crate::core::is_default;
use crate::core::merge_right::MergeRight;
use crate::core::variance::Invariant;

#[derive(
    Serialize, Deserialize, Clone, Debug, Default, Getters, PartialEq, Eq, JsonSchema, MergeRight,
)]
pub struct StaticConfig {
    ///
    /// Dictates how the server behaves and helps tune tailcall for all ingress
    /// requests. Features such as request batching, SSL, HTTP2 etc. can be
    /// configured here.
    pub server: ServerStatic,

    ///
    /// Dictates how tailcall should handle upstream requests/responses.
    /// Tuning upstream can improve performance and reliability for connections.
    pub upstream: UpstreamStatic,

    ///
    /// Linked files, that merge with config, schema or provide metadata.
    pub links: Vec<LinkStatic>,

    /// Enable [opentelemetry](https://opentelemetry.io) support.
    #[serde(default, skip_serializing_if = "is_default")]
    pub telemetry: TelemetryStatic,
}

impl StaticConfig {
    pub fn port(&self) -> u16 {
        self.server.port.unwrap_or(8000)
    }

    pub fn to_yaml(&self) -> anyhow::Result<String> {
        Ok(serde_yaml::to_string(self)?)
    }

    pub fn to_json(&self, pretty: bool) -> anyhow::Result<String> {
        if pretty {
            Ok(serde_json::to_string_pretty(self)?)
        } else {
            Ok(serde_json::to_string(self)?)
        }
    }

    pub fn from_json(json: &str) -> anyhow::Result<Self> {
        Ok(serde_json::from_str(json)?)
    }

    pub fn from_yaml(yaml: &str) -> anyhow::Result<Self> {
        Ok(serde_yaml::from_str(yaml)?)
    }

    pub fn from_graphql(graphql: &str) -> anyhow::Result<Self> {
        let config = Config::from_sdl(graphql).to_result()?;
        Ok(Self {
            server: ServerStatic::from(config.server),
            links: config.links.into_iter().map(LinkStatic::from).collect(),
            upstream: UpstreamStatic::from(config.upstream),
            telemetry: TelemetryStatic::from(config.telemetry),
        })
    }

    pub fn from_source(source: Source, data: &str) -> anyhow::Result<Self> {
        match source {
            Source::Json => Ok(StaticConfig::from_json(data)?),
            Source::Yml => Ok(StaticConfig::from_yaml(data)?),
            Source::GraphQL => Ok(StaticConfig::from_graphql(data)?),
        }
    }
}

impl Invariant for StaticConfig {
    fn unify(self, other: Self) -> Valid<Self, String> {
        Valid::succeed(self.merge_right(other))
    }
}

#[cfg(test)]
mod tests {
    // TODO: FIXME
}
