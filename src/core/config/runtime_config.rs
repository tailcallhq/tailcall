use derive_setters::Setters;
use serde::{Deserialize, Serialize};
use tailcall_macros::MergeRight;
use crate::core::config::{Link, Server, Telemetry, Upstream};

#[derive(
    Serialize,
    Deserialize,
    Clone,
    Debug,
    Default,
    Setters,
    PartialEq,
    Eq,
    schemars::JsonSchema,
    MergeRight,
)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeConfig {
    ///
    /// Dictates how the server behaves and helps tune tailcall for all ingress
    /// requests. Features such as request batching, SSL, HTTP2 etc. can be
    /// configured here.
    #[serde(default)]
    pub server: Server,

    ///
    /// Dictates how tailcall should handle upstream requests/responses.
    /// Tuning upstream can improve performance and reliability for connections.
    #[serde(default)]
    pub upstream: Upstream,

    ///
    /// A list of all links in the schema.
    #[serde(default, skip_serializing_if = "is_default")]
    pub links: Vec<Link>,

    /// Enable [opentelemetry](https://opentelemetry.io) support
    #[serde(default, skip_serializing_if = "is_default")]
    pub telemetry: Telemetry,
}

impl RuntimeConfig {
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
    /// Renders current config to graphQL string
    pub fn to_sdl(&self) -> String {
        crate::core::document::print(self.into())
    }

}