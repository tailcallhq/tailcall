use std::collections::BTreeMap;
use std::net::{AddrParseError, IpAddr};

use derive_setters::Setters;
use hyper::header::{HeaderName, HeaderValue};
use hyper::HeaderMap;

use crate::config::{self, HttpVersion};
use crate::valid::{Valid, ValidationError};

#[derive(Clone, Debug, Setters)]
pub struct Server {
  pub enable_apollo_tracing: bool,
  pub enable_cache_control_header: bool,
  pub enable_graphiql: bool,
  pub enable_introspection: bool,
  pub enable_query_validation: bool,
  pub enable_response_validation: bool,
  pub global_response_timeout: i64,
  pub worker: usize,
  pub port: u16,
  pub hostname: IpAddr,
  pub vars: BTreeMap<String, String>,
  pub response_headers: HeaderMap,
  pub http: HttpServer,
}

#[derive(Clone, Debug)]
pub enum HttpServer {
  HTTP1,
  HTTP2 { cert: String, key: String },
}

impl Default for Server {
  fn default() -> Self {
    // NOTE: Using unwrap because try_from default will never fail
    Server::try_from(config::Server::default()).unwrap()
  }
}

impl Server {
  pub fn get_enable_http_validation(&self) -> bool {
    self.enable_response_validation
  }
  pub fn get_enable_cache_control(&self) -> bool {
    self.enable_cache_control_header
  }

  pub fn get_enable_introspection(&self) -> bool {
    self.enable_introspection
  }

  pub fn get_enable_query_validation(&self) -> bool {
    self.enable_query_validation
  }
}

impl TryFrom<crate::config::Server> for Server {
  type Error = ValidationError<String>;

  fn try_from(server: config::Server) -> Result<Self, Self::Error> {
    let http_server = match server.clone().get_version() {
      HttpVersion::HTTP2 => {
        let cert = Valid::from_option(server.cert.clone(), "Certificate is required for HTTP2".to_string());
        let key = Valid::from_option(server.key.clone(), "Key is required for HTTP2".to_string());

        cert.zip(key).map(|(cert, key)| HttpServer::HTTP2 { cert, key })
      }
      _ => Valid::succeed(HttpServer::HTTP1),
    };

    validate_hostname(server.clone().get_hostname().to_lowercase())
      .zip(http_server)
      .zip(handle_response_headers((server).get_response_headers().0))
      .map(|((hostname, http), response_headers)| Server {
        enable_apollo_tracing: (server).enable_apollo_tracing(),
        enable_cache_control_header: (server).enable_cache_control(),
        enable_graphiql: (server).enable_graphiql(),
        enable_introspection: (server).enable_introspection(),
        enable_query_validation: (server).enable_query_validation(),
        enable_response_validation: (server).enable_http_validation(),
        global_response_timeout: (server).get_global_response_timeout(),
        http,
        worker: (server).get_workers(),
        port: (server).get_port(),
        hostname,
        vars: (server).get_vars(),
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
  use crate::config;

  #[test]
  fn test_try_from_default() {
    let actual = super::Server::try_from(config::Server::default());
    assert!(actual.is_ok())
  }
}
