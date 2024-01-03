use std::collections::BTreeMap;
use std::net::{AddrParseError, IpAddr};

use hyper::header::{HeaderName, HeaderValue};
use hyper::HeaderMap;
use serde::{Deserialize, Serialize};
use tc_core::blueprint::{is_default, Http, HttpVersion, KeyValues};
use tc_core::valid::{Valid, ValidationError};

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Server {
  #[serde(default, skip_serializing_if = "is_default")]
  pub apollo_tracing: Option<bool>,
  #[serde(default, skip_serializing_if = "is_default")]
  pub cache_control_header: Option<bool>,
  #[serde(default, skip_serializing_if = "is_default")]
  pub graphiql: Option<bool>,
  #[serde(default, skip_serializing_if = "is_default")]
  pub introspection: Option<bool>,
  #[serde(default, skip_serializing_if = "is_default")]
  pub query_validation: Option<bool>,
  #[serde(default, skip_serializing_if = "is_default")]
  pub response_validation: Option<bool>,
  #[serde(default, skip_serializing_if = "is_default")]
  pub batch_requests: Option<bool>,
  #[serde(default, skip_serializing_if = "is_default")]
  pub global_response_timeout: Option<i64>,
  #[serde(default, skip_serializing_if = "is_default")]
  pub workers: Option<usize>,
  #[serde(default, skip_serializing_if = "is_default")]
  pub hostname: Option<String>,
  #[serde(default, skip_serializing_if = "is_default")]
  pub port: Option<u16>,
  #[serde(default, skip_serializing_if = "is_default")]
  pub vars: KeyValues,
  #[serde(skip_serializing_if = "is_default", default)]
  pub response_headers: KeyValues,
  #[serde(default, skip_serializing_if = "is_default")]
  pub version: Option<HttpVersion>,
  #[serde(default, skip_serializing_if = "is_default")]
  pub cert: Option<String>,
  #[serde(default, skip_serializing_if = "is_default")]
  pub key: Option<String>,
  #[serde(default, skip_serializing_if = "is_default")]
  pub pipeline_flush: Option<bool>,
}

impl Server {
  pub fn enable_apollo_tracing(&self) -> bool {
    self.apollo_tracing.unwrap_or(false)
  }
  pub fn enable_graphiql(&self) -> bool {
    self.graphiql.unwrap_or(false)
  }
  pub fn get_global_response_timeout(&self) -> i64 {
    self.global_response_timeout.unwrap_or(0)
  }

  pub fn get_workers(&self) -> usize {
    self.workers.unwrap_or(num_cpus::get())
  }

  pub fn get_port(&self) -> u16 {
    self.port.unwrap_or(8000)
  }
  pub fn enable_http_validation(&self) -> bool {
    self.response_validation.unwrap_or(false)
  }
  pub fn enable_cache_control(&self) -> bool {
    self.cache_control_header.unwrap_or(false)
  }
  pub fn enable_introspection(&self) -> bool {
    self.introspection.unwrap_or(true)
  }
  pub fn enable_query_validation(&self) -> bool {
    self.query_validation.unwrap_or(false)
  }
  pub fn enable_batch_requests(&self) -> bool {
    self.batch_requests.unwrap_or(false)
  }

  pub fn get_hostname(&self) -> String {
    self.hostname.clone().unwrap_or("127.0.0.1".to_string())
  }

  pub fn get_vars(&self) -> BTreeMap<String, String> {
    self.vars.clone().0
  }

  pub fn get_response_headers(&self) -> KeyValues {
    self.response_headers.clone()
  }

  pub fn get_version(self) -> HttpVersion {
    self.version.unwrap_or(HttpVersion::HTTP1)
  }

  pub fn get_pipeline_flush(&self) -> bool {
    self.pipeline_flush.unwrap_or(true)
  }

  pub fn merge_right(mut self, other: Self) -> Self {
    self.apollo_tracing = other.apollo_tracing.or(self.apollo_tracing);
    self.cache_control_header = other.cache_control_header.or(self.cache_control_header);
    self.graphiql = other.graphiql.or(self.graphiql);
    self.introspection = other.introspection.or(self.introspection);
    self.query_validation = other.query_validation.or(self.query_validation);
    self.response_validation = other.response_validation.or(self.response_validation);
    self.batch_requests = other.batch_requests.or(self.batch_requests);
    self.global_response_timeout = other.global_response_timeout.or(self.global_response_timeout);
    self.workers = other.workers.or(self.workers);
    self.port = other.port.or(self.port);
    self.hostname = other.hostname.or(self.hostname);
    let mut vars = self.vars.0.clone();
    vars.extend(other.vars.0);
    self.vars = KeyValues(vars);
    let mut response_headers = self.response_headers.0.clone();
    response_headers.extend(other.response_headers.0);
    self.response_headers = KeyValues(response_headers);
    self.version = other.version.or(self.version);
    self.cert = other.cert.or(self.cert);
    self.key = other.key.or(self.key);
    self.pipeline_flush = other.pipeline_flush.or(self.pipeline_flush);
    self
  }
}

impl TryInto<tc_core::blueprint::Server> for Server {
  type Error = ValidationError<String>;

  fn try_into(self) -> Result<tc_core::blueprint::Server, Self::Error> {
    let http_server = match self.clone().get_version() {
      HttpVersion::HTTP2 => {
        let cert = Valid::from_option(self.cert.clone(), "Certificate is required for HTTP2".to_string());
        let key = Valid::from_option(self.key.clone(), "Key is required for HTTP2".to_string());

        cert.zip(key).map(|(cert, key)| Http::HTTP2 { cert, key })
      }
      _ => Valid::succeed(Http::HTTP1),
    };

    validate_hostname((self).get_hostname().to_lowercase())
      .zip(http_server)
      .zip(handle_response_headers((self).get_response_headers().0))
      .map(|((hostname, http), response_headers)| tc_core::blueprint::Server {
        enable_apollo_tracing: (self).enable_apollo_tracing(),
        enable_cache_control_header: (self).enable_cache_control(),
        enable_graphiql: (self).enable_graphiql(),
        enable_introspection: (self).enable_introspection(),
        enable_query_validation: (self).enable_query_validation(),
        enable_response_validation: (self).enable_http_validation(),
        enable_batch_requests: (self).enable_batch_requests(),
        global_response_timeout: (self).get_global_response_timeout(),
        http,
        worker: (self).get_workers(),
        port: (self).get_port(),
        hostname,
        vars: (self).get_vars(),
        pipeline_flush: (self).get_pipeline_flush(),
        response_headers,
      })
      .to_result()
  }
}

fn validate_hostname(hostname: String) -> Valid<IpAddr, String> {
  if hostname == "localhost" {
    Valid::succeed(IpAddr::from([127, 0, 0, 1]))
  } else {
    Valid::from(
      hostname
        .parse()
        .map_err(|e: AddrParseError| ValidationError::new(format!("Parsing failed because of {}", e))),
    )
    .trace("hostname")
    .trace("@server")
    .trace("schema")
  }
}

fn handle_response_headers(resp_headers: BTreeMap<String, String>) -> Valid<HeaderMap, String> {
  Valid::from_iter(resp_headers.iter(), |(k, v)| {
    let name = Valid::from(
      HeaderName::from_bytes(k.as_bytes())
        .map_err(|e| ValidationError::new(format!("Parsing failed because of {}", e))),
    );
    let value = Valid::from(
      HeaderValue::from_str(v.as_str()).map_err(|e| ValidationError::new(format!("Parsing failed because of {}", e))),
    );
    name.zip(value)
  })
  .map(|headers| headers.into_iter().collect::<HeaderMap>())
  .trace("responseHeaders")
  .trace("@server")
  .trace("schema")
}

#[cfg(test)]
mod tests {
  use tc_core::valid::ValidationError;

  use crate::config;

  #[test]
  fn test_try_from_default() {
    let actual: Result<tc_core::blueprint::Server, ValidationError<String>> = config::Server::default().try_into();
    assert!(actual.is_ok())
  }
}
