use std::collections::BTreeMap;
use std::net::{AddrParseError, IpAddr};

use derive_setters::Setters;
use hyper::header::{HeaderName, HeaderValue};
use hyper::HeaderMap;

use crate::config;
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
  pub port: u16,
  pub hostname: IpAddr,
  pub vars: BTreeMap<String, String>,
  pub response_headers: HeaderMap,
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

  fn try_from(config_server: config::Server) -> Result<Self, Self::Error> {
    configure_server(&config_server).to_result()
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

fn configure_server(config_config: &config::Server) -> Valid<Server, String> {
  validate_hostname(config_config.get_hostname().to_lowercase())
    .zip(handle_response_headers(config_config.get_response_headers().0))
    .map(|(hostname, response_headers)| Server {
      enable_apollo_tracing: config_config.enable_apollo_tracing(),
      enable_cache_control_header: config_config.enable_cache_control(),
      enable_graphiql: config_config.enable_graphiql(),
      enable_introspection: config_config.enable_introspection(),
      enable_query_validation: config_config.enable_query_validation(),
      enable_response_validation: config_config.enable_http_validation(),
      global_response_timeout: config_config.get_global_response_timeout(),
      port: config_config.get_port(),
      hostname,
      vars: config_config.get_vars(),
      response_headers,
    })
}
