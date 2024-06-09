use std::collections::BTreeSet;

use derive_setters::Setters;
use serde::{Deserialize, Serialize};

use crate::core::is_default;
use crate::core::macros::MergeRight;
use crate::core::merge_right::MergeRight;

use crate::core::config::positioned_config::PositionedConfig;
use crate::core::macros::PositionedConfig;

use super::position::Pos;

const DEFAULT_MAX_SIZE: usize = 100;

#[derive(
    Serialize, Deserialize, PartialEq, Eq, Clone, Debug, Setters, schemars::JsonSchema, MergeRight,
)]
#[serde(rename_all = "camelCase", default)]
pub struct Batch {
    pub delay: usize,
    pub headers: BTreeSet<String>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub max_size: Option<usize>,
}
impl Default for Batch {
    fn default() -> Self {
        Batch {
            max_size: Some(DEFAULT_MAX_SIZE),
            delay: 0,
            headers: BTreeSet::new(),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, schemars::JsonSchema, MergeRight)]
pub struct Proxy {
    pub url: String,
}

#[derive(
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Clone,
    Debug,
    Setters,
    Default,
    schemars::JsonSchema,
    MergeRight,
    PositionedConfig,
)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase", default)]
/// The `upstream` directive allows you to control various aspects of the
/// upstream server connection. This includes settings like connection timeouts,
/// keep-alive intervals, and more. If not specified, default values are used.
pub struct Upstream {
    #[serde(rename = "onRequest", default, skip_serializing_if = "is_default")]
    /// onRequest field gives the ability to specify the global request
    /// interception handler.
    pub on_request: Option<String>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// `allowedHeaders` defines the HTTP headers allowed to be forwarded to
    /// upstream services. If not set, no headers are forwarded, enhancing
    /// security but possibly limiting data flow.
    #[positioned_field(option_field)]
    pub allowed_headers: Option<Pos<BTreeSet<String>>>,

    #[serde(rename = "baseURL", default, skip_serializing_if = "is_default")]
    /// This refers to the default base URL for your APIs. If it's not
    /// explicitly mentioned in the `@upstream` operator, then each
    /// [@http](#http) operator must specify its own `baseURL`. If neither
    /// `@upstream` nor [@http](#http) provides a `baseURL`, it results in a
    /// compilation error.
    #[positioned_field(option_field)]
    pub base_url: Option<Pos<String>>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// An object that specifies the batch settings, including `maxSize` (the
    /// maximum size of the batch), `delay` (the delay in milliseconds between
    /// each batch), and `headers` (an array of HTTP headers to be included in
    /// the batch).
    #[positioned_field(option_field)]
    pub batch: Option<Pos<Batch>>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// The time in seconds that the connection will wait for a response before
    /// timing out.
    #[positioned_field(option_field)]
    pub connect_timeout: Option<Pos<u64>>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// Providing httpCache size enables Tailcall's HTTP caching, adhering to the [HTTP Caching RFC](https://tools.ietf.org/html/rfc7234), to enhance performance by minimizing redundant data fetches. Defaults to `0` if unspecified.
    #[positioned_field(option_field)]
    pub http_cache: Option<Pos<u64>>,

    #[setters(strip_option)]
    #[serde(rename = "http2Only", default, skip_serializing_if = "is_default")]
    /// The `http2Only` setting allows you to specify whether the client should
    /// always issue HTTP2 requests, without checking if the server supports it
    /// or not. By default it is set to `false` for all HTTP requests made by
    /// the server, but is automatically set to true for GRPC.
    #[positioned_field(option_field)]
    pub http_2_only: Option<Pos<bool>>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// The time in seconds between each keep-alive message sent to maintain the
    /// connection.
    #[positioned_field(option_field)]
    pub keep_alive_interval: Option<Pos<u64>>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// The time in seconds that the connection will wait for a keep-alive
    /// message before closing.
    #[positioned_field(option_field)]
    pub keep_alive_timeout: Option<Pos<u64>>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// A boolean value that determines whether keep-alive messages should be
    /// sent while the connection is idle.
    #[positioned_field(option_field)]
    pub keep_alive_while_idle: Option<Pos<bool>>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// The maximum number of idle connections that will be maintained per host.
    #[positioned_field(option_field)]
    pub pool_max_idle_per_host: Option<Pos<usize>>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// The time in seconds that the connection pool will wait before closing
    /// idle connections.
    #[positioned_field(option_field)]
    pub pool_idle_timeout: Option<Pos<u64>>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// The `proxy` setting defines an intermediary server through which the
    /// upstream requests will be routed before reaching their intended
    /// endpoint. By specifying a proxy URL, you introduce an additional layer,
    /// enabling custom routing and security policies.
    #[positioned_field(option_field)]
    pub proxy: Option<Pos<Proxy>>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// The time in seconds between each TCP keep-alive message sent to maintain
    /// the connection.
    #[positioned_field(option_field)]
    pub tcp_keep_alive: Option<Pos<u64>>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// The maximum time in seconds that the connection will wait for a
    /// response.
    #[positioned_field(option_field)]
    pub timeout: Option<Pos<u64>>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// The User-Agent header value to be used in HTTP requests. @default
    /// `Tailcall/1.0`
    #[positioned_field(option_field)]
    pub user_agent: Option<Pos<String>>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// When set to `true`, it will ensure no HTTP, GRPC, or any other IO call
    /// is made more than once within the context of a single GraphQL request.
    #[positioned_field(option_field)]
    pub dedupe: Option<Pos<bool>>,
}

impl Upstream {
    pub fn get_pool_idle_timeout(&self) -> u64 {
        self.pool_idle_timeout
            .as_ref()
            .map_or(60, |Pos { inner, .. }| *inner)
    }

    pub fn get_pool_max_idle_per_host(&self) -> usize {
        self.pool_max_idle_per_host
            .as_ref()
            .map_or(60, |Pos { inner, .. }| *inner)
    }

    pub fn get_keep_alive_interval(&self) -> u64 {
        self.keep_alive_interval
            .as_ref()
            .map_or(60, |Pos { inner, .. }| *inner)
    }

    pub fn get_keep_alive_timeout(&self) -> u64 {
        self.keep_alive_timeout
            .as_ref()
            .map_or(60, |Pos { inner, .. }| *inner)
    }

    pub fn get_keep_alive_while_idle(&self) -> bool {
        self.keep_alive_while_idle
            .as_ref()
            .map_or(false, |Pos { inner, .. }| *inner)
    }

    pub fn get_connect_timeout(&self) -> u64 {
        self.connect_timeout
            .as_ref()
            .map_or(60, |Pos { inner, .. }| *inner)
    }

    pub fn get_timeout(&self) -> u64 {
        self.timeout.as_ref().map_or(60, |Pos { inner, .. }| *inner)
    }

    pub fn get_tcp_keep_alive(&self) -> u64 {
        self.tcp_keep_alive
            .as_ref()
            .map_or(5, |Pos { inner, .. }| *inner)
    }

    pub fn get_user_agent(&self) -> String {
        self.user_agent
            .clone()
            .map_or("Tailcall/1.0".to_string(), |Pos { inner, .. }| inner)
    }

    pub fn get_http_cache_size(&self) -> u64 {
        self.http_cache
            .as_ref()
            .map_or(0, |Pos { inner, .. }| *inner)
    }

    pub fn get_allowed_headers(&self) -> BTreeSet<String> {
        self.allowed_headers.clone().unwrap_or_default().inner
    }

    pub fn get_delay(&self) -> usize {
        self.batch.clone().unwrap_or_default().delay
    }

    pub fn get_max_size(&self) -> usize {
        self.batch
            .as_ref()
            .map_or(DEFAULT_MAX_SIZE, |b| b.max_size.unwrap_or(DEFAULT_MAX_SIZE))
    }

    pub fn get_http_2_only(&self) -> bool {
        self.http_2_only
            .as_ref()
            .map_or(false, |Pos { inner, .. }| *inner)
    }

    pub fn get_dedupe(&self) -> bool {
        self.dedupe
            .as_ref()
            .map_or(false, |Pos { inner, .. }| *inner)
    }

    pub fn get_on_request(&self) -> Option<String> {
        self.on_request.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_upstream_with_headers(headers: &[&str]) -> Upstream {
        Upstream {
            allowed_headers: Some(Pos::new(
                0,
                0,
                None,
                headers.iter().map(|s| s.to_string()).collect(),
            )),
            ..Default::default()
        }
    }

    #[test]
    fn allowed_headers_merge_both() {
        let a = setup_upstream_with_headers(&["a", "b", "c"]);
        let b = setup_upstream_with_headers(&["d", "e", "f"]);
        let merged = a.merge_right(b);
        assert_eq!(
            merged.allowed_headers,
            Some(Pos::new(
                0,
                0,
                None,
                ["a", "b", "c", "d", "e", "f"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect()
            ))
        );
    }

    #[test]
    fn allowed_headers_merge_first() {
        let a = setup_upstream_with_headers(&["a", "b", "c"]);
        let b = Upstream::default();
        let merged = a.merge_right(b);

        assert_eq!(
            merged.allowed_headers,
            Some(Pos::new(
                0,
                0,
                None,
                ["a", "b", "c"].iter().map(|s| s.to_string()).collect()
            ))
        );
    }

    #[test]
    fn allowed_headers_merge_second() {
        let a = Upstream::default();
        let b = setup_upstream_with_headers(&["a", "b", "c"]);
        let merged = a.merge_right(b);

        assert_eq!(
            merged.allowed_headers,
            Some(Pos::new(
                0,
                0,
                None,
                ["a", "b", "c"].iter().map(|s| s.to_string()).collect()
            ))
        );
    }
}
