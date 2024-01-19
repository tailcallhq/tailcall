use std::collections::BTreeMap;
use std::net::{AddrParseError, IpAddr};

use derive_setters::Setters;
use hyper::header::{HeaderName, HeaderValue};
use hyper::HeaderMap;

use super::init_context::InitContext;
use super::Auth;
use crate::config::{self, HttpVersion};
use crate::directive::DirectiveCodec;
use crate::valid::{Valid, ValidationError};

#[derive(Clone, Debug, Setters)]
pub struct Server {
  pub enable_apollo_tracing: bool,
  pub enable_cache_control_header: bool,
  pub enable_graphiql: bool,
  pub enable_introspection: bool,
  pub enable_query_validation: bool,
  pub enable_response_validation: bool,
  pub enable_batch_requests: bool,
  pub global_response_timeout: i64,
  pub worker: usize,
  pub port: u16,
  pub hostname: IpAddr,
  pub vars: BTreeMap<String, String>,
  pub response_headers: HeaderMap,
  pub http: Http,
  pub pipeline_flush: bool,
  pub auth: Auth,
}

#[derive(Clone, Debug)]
pub enum Http {
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

impl TryFrom<config::Server> for Server {
  type Error = ValidationError<String>;

  fn try_from(config_server: config::Server) -> Result<Self, Self::Error> {
    let http_server = match config_server.clone().get_version() {
      HttpVersion::HTTP2 => {
        let cert = Valid::from_option(
          config_server.cert.clone(),
          "Certificate is required for HTTP2".to_string(),
        );
        let key = Valid::from_option(config_server.key.clone(), "Key is required for HTTP2".to_string());

        cert.zip(key).map(|(cert, key)| Http::HTTP2 { cert, key })
      }
      _ => Valid::succeed(Http::HTTP1),
    };
    let init_context = InitContext::from(&config_server);

    validate_hostname((config_server).get_hostname().to_lowercase())
      .zip(http_server)
      .zip(handle_response_headers((config_server).get_response_headers().0))
      .zip(Auth::make(&init_context, &config_server.auth))
      .map(|(((hostname, http), response_headers), auth)| Server {
        enable_apollo_tracing: (config_server).enable_apollo_tracing(),
        enable_cache_control_header: (config_server).enable_cache_control(),
        enable_graphiql: (config_server).enable_graphiql(),
        enable_introspection: (config_server).enable_introspection(),
        enable_query_validation: (config_server).enable_query_validation(),
        enable_response_validation: (config_server).enable_http_validation(),
        enable_batch_requests: (config_server).enable_batch_requests(),
        global_response_timeout: (config_server).get_global_response_timeout(),
        http,
        worker: (config_server).get_workers(),
        port: (config_server).get_port(),
        hostname,
        vars: (config_server).get_vars(),
        pipeline_flush: (config_server).get_pipeline_flush(),
        response_headers,
        auth,
      })
      .trace(config::Server::trace_name().as_str())
      .trace("schema")
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
