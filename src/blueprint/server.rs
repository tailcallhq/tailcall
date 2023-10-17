use std::collections::BTreeMap;
use std::net::{AddrParseError, IpAddr};

use derive_setters::Setters;
use hyper::header::{HeaderName, HeaderValue};
use hyper::HeaderMap;

use crate::config;
use crate::valid::{Valid, ValidExtensions, ValidationError, VectorExtension};

#[derive(Clone, Debug, Setters)]
pub struct Server {
  pub enable_apollo_tracing: bool,
  pub enable_cache_control_header: bool,
  pub enable_graphiql: Option<String>,
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

  fn try_from(config_server: config::Server) -> Valid<Self, String> {
    // Configure other server settings
    let server = configure_server(&config_server)?;
    Valid::Ok(server.clone())
  }
}

fn validate_hostname(hostname: String) -> Valid<IpAddr, String> {
  let host = if hostname == "localhost" {
    IpAddr::from([127, 0, 0, 1])
  } else {
    hostname
      .parse()
      .map_err(|e: AddrParseError| ValidationError::new(format!("Parsing failed because of {}", e)))
      .trace("hostname")
      .trace("@server")
      .trace("schema")?
  };
  Ok(host)
}

const RESTRICTED_ROUTES: &[&str] = &["/", "/graphql"];

fn handle_graphiql(graphiql: Option<String>) -> Valid<Option<String>, String> {
  let mut graph = None;
  if let Some(enable_graphiql) = graphiql.clone() {
    let lowered_route = enable_graphiql.to_lowercase();
    if RESTRICTED_ROUTES.contains(&lowered_route.as_str()) {
      return Err(
        ValidationError::new(format!(
          "Cannot use restricted routes '{}' for enabling graphiql",
          enable_graphiql
        ))
        .trace("enableGraphiql")
        .trace("@server")
        .trace("schema"),
      );
    } else {
      graph = Some(enable_graphiql);
    }
  };
  Ok(graph)
}

fn handle_response_headers(resp_headers: BTreeMap<String, String>) -> Valid<HeaderMap, String> {
  let headers = resp_headers
    .validate_all(|(k, v)| {
      let name = HeaderName::from_bytes(k.as_bytes())
        .map_err(|e| ValidationError::new(format!("Parsing failed because of {}", e)));
      let value =
        HeaderValue::from_str(v.as_str()).map_err(|e| ValidationError::new(format!("Parsing failed because of {}", e)));
      name.validate_both(value)
    })
    .trace("responseHeaders")
    .trace("@server")
    .trace("schema")?;

  let mut response_headers = HeaderMap::new();
  response_headers.extend(headers);
  Ok(response_headers)
}

fn configure_server(config_config: &config::Server) -> Valid<Server, String> {
  Ok(Server {
    enable_apollo_tracing: config_config.enable_apollo_tracing(),
    enable_cache_control_header: config_config.enable_cache_control(),
    enable_graphiql: handle_graphiql(config_config.enable_graphiql())?,
    enable_introspection: config_config.enable_introspection(),
    enable_query_validation: config_config.enable_query_validation(),
    enable_response_validation: config_config.enable_http_validation(),
    global_response_timeout: config_config.get_global_response_timeout(),
    port: config_config.get_port(),
    hostname: validate_hostname(config_config.get_hostname().to_lowercase())?,
    vars: config_config.get_vars(),
    response_headers: handle_response_headers(config_config.get_response_headers().0)?,
  })
}
