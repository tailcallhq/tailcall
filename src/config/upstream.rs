use std::collections::BTreeSet;

use derive_setters::Setters;
use serde::{Deserialize, Serialize};

use crate::is_default;
use crate::merge_right::MergeRight;

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, Setters, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct Batch {
    pub delay: usize,
    pub headers: BTreeSet<String>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub max_size: Option<usize>,
}
impl Default for Batch {
    fn default() -> Self {
        Batch { max_size: Some(100), delay: 0, headers: BTreeSet::new() }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, schemars::JsonSchema)]
pub struct Proxy {
    pub url: String,
}

impl MergeRight for Proxy {
    fn merge_right(self, other: Self) -> Self {
        other
    }
}

#[derive(
    Serialize, Deserialize, PartialEq, Eq, Clone, Debug, Setters, Default, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase", default)]
/// The `upstream` directive allows you to control various aspects of the
/// upstream server connection. This includes settings like connection timeouts,
/// keep-alive intervals, and more. If not specified, default values are used.
pub struct Upstream {
    #[serde(default, skip_serializing_if = "is_default")]
    /// `allowedHeaders` defines the HTTP headers allowed to be forwarded to
    /// upstream services. If not set, no headers are forwarded, enhancing
    /// security but possibly limiting data flow.
    pub allowed_headers: Option<BTreeSet<String>>,

    #[serde(rename = "baseURL", default, skip_serializing_if = "is_default")]
    /// This refers to the default base URL for your APIs. If it's not
    /// explicitly mentioned in the `@upstream` operator, then each
    /// [@http](#http) operator must specify its own `baseURL`. If neither
    /// `@upstream` nor [@http](#http) provides a `baseURL`, it results in a
    /// compilation error.
    pub base_url: Option<String>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// An object that specifies the batch settings, including `maxSize` (the
    /// maximum size of the batch), `delay` (the delay in milliseconds between
    /// each batch), and `headers` (an array of HTTP headers to be included in
    /// the batch).
    pub batch: Option<Batch>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// The time in seconds that the connection will wait for a response before
    /// timing out.
    pub connect_timeout: Option<u64>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// Activating this enables Tailcall's HTTP caching, adhering to the [HTTP Caching RFC](https://tools.ietf.org/html/rfc7234), to enhance performance by minimizing redundant data fetches. Defaults to `false` if unspecified.
    pub http_cache: Option<bool>,

    #[setters(strip_option)]
    #[serde(rename = "http2Only", default, skip_serializing_if = "is_default")]
    /// The `http2Only` setting allows you to specify whether the client should
    /// always issue HTTP2 requests, without checking if the server supports it
    /// or not. By default it is set to `false` for all HTTP requests made by
    /// the server, but is automatically set to true for GRPC.
    pub http2_only: Option<bool>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// The time in seconds between each keep-alive message sent to maintain the
    /// connection.
    pub keep_alive_interval: Option<u64>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// The time in seconds that the connection will wait for a keep-alive
    /// message before closing.
    pub keep_alive_timeout: Option<u64>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// A boolean value that determines whether keep-alive messages should be
    /// sent while the connection is idle.
    pub keep_alive_while_idle: Option<bool>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// The maximum number of idle connections that will be maintained per host.
    pub pool_max_idle_per_host: Option<usize>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// The time in seconds that the connection pool will wait before closing
    /// idle connections.
    pub pool_idle_timeout: Option<u64>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// The `proxy` setting defines an intermediary server through which the
    /// upstream requests will be routed before reaching their intended
    /// endpoint. By specifying a proxy URL, you introduce an additional layer,
    /// enabling custom routing and security policies.
    pub proxy: Option<Proxy>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// The time in seconds between each TCP keep-alive message sent to maintain
    /// the connection.
    pub tcp_keep_alive: Option<u64>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// The maximum time in seconds that the connection will wait for a
    /// response.
    pub timeout: Option<u64>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// The User-Agent header value to be used in HTTP requests. @default
    /// `Tailcall/1.0`
    pub user_agent: Option<String>,
}

impl Upstream {
    pub fn get_pool_idle_timeout(&self) -> u64 {
        self.pool_idle_timeout.unwrap_or(60)
    }
    pub fn get_pool_max_idle_per_host(&self) -> usize {
        self.pool_max_idle_per_host.unwrap_or(60)
    }
    pub fn get_keep_alive_interval(&self) -> u64 {
        self.keep_alive_interval.unwrap_or(60)
    }
    pub fn get_keep_alive_timeout(&self) -> u64 {
        self.keep_alive_timeout.unwrap_or(60)
    }
    pub fn get_keep_alive_while_idle(&self) -> bool {
        self.keep_alive_while_idle.unwrap_or(false)
    }
    pub fn get_connect_timeout(&self) -> u64 {
        self.connect_timeout.unwrap_or(60)
    }
    pub fn get_timeout(&self) -> u64 {
        self.timeout.unwrap_or(60)
    }
    pub fn get_tcp_keep_alive(&self) -> u64 {
        self.tcp_keep_alive.unwrap_or(5)
    }
    pub fn get_user_agent(&self) -> String {
        self.user_agent
            .clone()
            .unwrap_or("Tailcall/1.0".to_string())
    }
    pub fn get_enable_http_cache(&self) -> bool {
        self.http_cache.unwrap_or(false)
    }
    pub fn get_allowed_headers(&self) -> BTreeSet<String> {
        self.allowed_headers.clone().unwrap_or_default()
    }
    pub fn get_delay(&self) -> usize {
        self.batch.clone().unwrap_or_default().delay
    }

    pub fn get_max_size(&self) -> usize {
        self.batch.clone()
            .unwrap_or_default()
            .max_size
            .unwrap_or(100)
    }

    pub fn get_http_2_only(&self) -> bool {
        self.http2_only.unwrap_or(false)
    }
}

impl MergeRight for Upstream {
    // TODO: add unit tests for merge
    fn merge_right(mut self, other: Self) -> Self {
        self.allowed_headers = self.allowed_headers.merge_right(other.allowed_headers);
        self.base_url = self.base_url.merge_right(other.base_url);
        self.connect_timeout = self.connect_timeout.merge_right(other.connect_timeout);
        self.http_cache = self.http_cache.merge_right(other.http_cache);
        self.keep_alive_interval = self
            .keep_alive_interval
            .merge_right(other.keep_alive_interval);
        self.keep_alive_timeout = self
            .keep_alive_timeout
            .merge_right(other.keep_alive_timeout);
        self.keep_alive_while_idle = self
            .keep_alive_while_idle
            .merge_right(other.keep_alive_while_idle);
        self.pool_idle_timeout = self.pool_idle_timeout.merge_right(other.pool_idle_timeout);
        self.pool_max_idle_per_host = self
            .pool_max_idle_per_host
            .merge_right(other.pool_max_idle_per_host);
        self.proxy = self.proxy.merge_right(other.proxy);
        self.tcp_keep_alive = self.tcp_keep_alive.merge_right(other.tcp_keep_alive);
        self.timeout = self.timeout.merge_right(other.timeout);
        self.user_agent = self.user_agent.merge_right(other.user_agent);

        if let Some(other) = other.batch {
            let mut batch = self.batch.unwrap_or_default();
            batch.max_size = other.max_size;
            batch.delay = other.delay;
            batch.headers = batch.headers.merge_right(other.headers);

            self.batch = Some(batch);
        }

        self.http2_only = self.http2_only.merge_right(other.http2_only);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_upstream_with_headers(headers: &[&str]) -> Upstream {
        Upstream {
            allowed_headers: Some(headers.iter().map(|s| s.to_string()).collect()),
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
            Some(
                ["a", "b", "c", "d", "e", "f"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect()
            )
        );
    }

    #[test]
    fn allowed_headers_merge_first() {
        let a = setup_upstream_with_headers(&["a", "b", "c"]);
        let b = Upstream::default();
        let merged = a.merge_right(b);

        assert_eq!(
            merged.allowed_headers,
            Some(["a", "b", "c"].iter().map(|s| s.to_string()).collect())
        );
    }

    #[test]
    fn allowed_headers_merge_second() {
        let a = Upstream::default();
        let b = setup_upstream_with_headers(&["a", "b", "c"]);
        let merged = a.merge_right(b);

        assert_eq!(
            merged.allowed_headers,
            Some(["a", "b", "c"].iter().map(|s| s.to_string()).collect())
        );
    }
}
