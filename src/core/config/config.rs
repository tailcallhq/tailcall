use derive_getters::Getters;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tailcall_macros::MergeRight;
use tailcall_valid::Valid;

use super::{LinkConfig, ServerConfig, Source, TelemetryConfig, UpstreamConfig};
use crate::core::{is_default, merge_right::MergeRight, variance::Invariant};

#[derive(
    Serialize, Deserialize, Clone, Debug, Default, Getters, PartialEq, Eq, JsonSchema, MergeRight,
)]
pub struct Config {
    ///
    /// Dictates how the server behaves and helps tune tailcall for all ingress
    /// requests. Features such as request batching, SSL, HTTP2 etc. can be
    /// configured here.
    pub server: ServerConfig,

    ///
    /// Dictates how tailcall should handle upstream requests/responses.
    /// Tuning upstream can improve performance and reliability for connections.
    pub upstream: UpstreamConfig,

    ///
    /// Linked files, that merge with config, schema or provide metadata.
    pub links: Vec<LinkConfig>,

    /// Enable [opentelemetry](https://opentelemetry.io) support.
    #[serde(default, skip_serializing_if = "is_default")]
    pub telemetry: TelemetryConfig,
}

impl Config {
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

    pub fn from_source(source: Source, data: &str) -> anyhow::Result<Self> {
        match source {
            Source::Json => Ok(Config::from_json(data)?),
            Source::Yml => Ok(Config::from_yaml(data)?),
        }
    }
}

impl Invariant for Config {
    fn unify(self, other: Self) -> Valid<Self, String> {
        Valid::succeed(self.merge_right(other))
    }
}

#[cfg(test)]
mod tests {
   // TODO: FIXME
}
