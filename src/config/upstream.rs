use std::collections::BTreeSet;

use derive_setters::Setters;
use serde::{Deserialize, Serialize};

use crate::is_default;

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, Setters, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct Batch {
  pub max_size: usize,
  pub delay: usize,
  pub headers: BTreeSet<String>,
}
impl Default for Batch {
  fn default() -> Self {
    Batch { max_size: 100, delay: 0, headers: BTreeSet::new() }
  }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, schemars::JsonSchema)]
pub struct Proxy {
  pub url: String,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, Setters, Default, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", default)]
/// The `upstream` directive allows you to control various aspects of the upstream server connection. This includes settings like connection timeouts, keep-alive intervals, and more. If not specified, default values are used.
pub struct Upstream {
  #[serde(default, skip_serializing_if = "is_default")]
  /// The time in seconds that the connection pool will wait before closing idle connections.
  pub pool_idle_timeout: Option<u64>,
  #[serde(default, skip_serializing_if = "is_default")]
  /// The maximum number of idle connections that will be maintained per host.
  pub pool_max_idle_per_host: Option<usize>,
  #[serde(default, skip_serializing_if = "is_default")]
  /// The time in seconds between each keep-alive message sent to maintain the connection.
  pub keep_alive_interval: Option<u64>,
  #[serde(default, skip_serializing_if = "is_default")]
  /// The time in seconds that the connection will wait for a keep-alive message before closing.
  pub keep_alive_timeout: Option<u64>,
  #[serde(default, skip_serializing_if = "is_default")]
  /// A boolean value that determines whether keep-alive messages should be sent while the connection is idle.
  pub keep_alive_while_idle: Option<bool>,
  #[serde(default, skip_serializing_if = "is_default")]
  /// The `proxy` setting defines an intermediary server through which the upstream requests will be routed before reaching their intended endpoint. By specifying a proxy URL, you introduce an additional layer, enabling custom routing and security policies.
  pub proxy: Option<Proxy>,
  #[serde(default, skip_serializing_if = "is_default")]
  /// The time in seconds that the connection will wait for a response before timing out.
  pub connect_timeout: Option<u64>,
  #[serde(default, skip_serializing_if = "is_default")]
  /// The maximum time in seconds that the connection will wait for a response.
  pub timeout: Option<u64>,
  #[serde(default, skip_serializing_if = "is_default")]
  /// The time in seconds between each TCP keep-alive message sent to maintain the connection.
  pub tcp_keep_alive: Option<u64>,
  #[serde(default, skip_serializing_if = "is_default")]
  /// The User-Agent header value to be used in HTTP requests. @default `Tailcall/1.0`
  pub user_agent: Option<String>,
  #[serde(default, skip_serializing_if = "is_default")]
  /// `allowedHeaders` defines the HTTP headers allowed to be forwarded to upstream services. If not set, no headers are forwarded, enhancing security but possibly limiting data flow.
  pub allowed_headers: Option<BTreeSet<String>>,
  #[serde(rename = "baseURL", default, skip_serializing_if = "is_default")]
  /// This refers to the default base URL for your APIs. If it's not explicitly mentioned in the `@upstream` operator, then each [@http](#http) operator must specify its own `baseURL`. If neither `@upstream` nor [@http](#http) provides a `baseURL`, it results in a compilation error.
  pub base_url: Option<String>,
  #[serde(default, skip_serializing_if = "is_default")]
  /// Activating this enables Tailcall's HTTP caching, adhering to the [HTTP Caching RFC](https://tools.ietf.org/html/rfc7234), to enhance performance by minimizing redundant data fetches. Defaults to `false` if unspecified.
  pub http_cache: Option<bool>,
  #[serde(default, skip_serializing_if = "is_default")]
  /// An object that specifies the batch settings, including `maxSize` (the maximum size of the batch), `delay` (the delay in milliseconds between each batch), and `headers` (an array of HTTP headers to be included in the batch).
  pub batch: Option<Batch>,
  #[setters(strip_option)]
  #[serde(rename = "http2Only", default, skip_serializing_if = "is_default")]
  /// The `http2Only` setting allows you to specify whether the client should always issue HTTP2 requests, without checking if the server supports it or not. By default it is set to `false` for all HTTP requests made by the server, but is automatically set to true for GRPC.
  pub http2_only: Option<bool>,
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
    self.user_agent.clone().unwrap_or("Tailcall/1.0".to_string())
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
    self.batch.clone().unwrap_or_default().max_size
  }

  pub fn get_http_2_only(&self) -> bool {
    self.http2_only.unwrap_or(false)
  }

  // TODO: add unit tests for merge
  pub fn merge_right(mut self, other: Self) -> Self {
    self.allowed_headers = other.allowed_headers.map(|other| {
      if let Some(mut self_headers) = self.allowed_headers {
        self_headers.extend(&mut other.iter().map(|s| s.to_owned()));
        self_headers
      } else {
        other
      }
    });
    self.base_url = other.base_url.or(self.base_url);
    self.connect_timeout = other.connect_timeout.or(self.connect_timeout);
    self.http_cache = other.http_cache.or(self.http_cache);
    self.keep_alive_interval = other.keep_alive_interval.or(self.keep_alive_interval);
    self.keep_alive_timeout = other.keep_alive_timeout.or(self.keep_alive_timeout);
    self.keep_alive_while_idle = other.keep_alive_while_idle.or(self.keep_alive_while_idle);
    self.pool_idle_timeout = other.pool_idle_timeout.or(self.pool_idle_timeout);
    self.pool_max_idle_per_host = other.pool_max_idle_per_host.or(self.pool_max_idle_per_host);
    self.proxy = other.proxy.or(self.proxy);
    self.tcp_keep_alive = other.tcp_keep_alive.or(self.tcp_keep_alive);
    self.timeout = other.timeout.or(self.timeout);
    self.user_agent = other.user_agent.or(self.user_agent);

    if let Some(other) = other.batch {
      let mut batch = self.batch.unwrap_or_default();
      batch.max_size = other.max_size;
      batch.delay = other.delay;
      batch.headers.extend(other.headers);

      self.batch = Some(batch);
    }

    self.http2_only = other.http2_only.or(self.http2_only);
    self
  }
}
