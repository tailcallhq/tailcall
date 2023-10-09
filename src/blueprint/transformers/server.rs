use hyper::header::{HeaderName, HeaderValue};
use hyper::HeaderMap;

use crate::blueprint::blueprint;
use crate::blueprint::transform::Transform;
use crate::blueprint::transformers::Valid;
use crate::config::{self};
use crate::valid::{ValidExtensions, ValidationError, VectorExtension};

const RESTRICTED_ROUTES: &[&str] = &["/", "/graphql"];

/// Handle the graphql in the [`config::Server`]
pub struct ServerGraphqlTransform;

impl From<ServerGraphqlTransform> for Transform<config::Server, blueprint::Server, String> {
  fn from(_value: ServerGraphqlTransform) -> Self {
    Transform::new(move |config_server: &config::Server, mut server: blueprint::Server| {
      if let Some(enable_graphiql) = config_server.enable_graphiql.clone() {
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
          server = server.clone().enable_graphiql(enable_graphiql);
        }
      }
      Ok(server)
    })
  }
}

/// Handle the responce headers in [`config::Server`]
pub struct ServerResponseHeaderTransform;

impl From<ServerResponseHeaderTransform> for Transform<config::Server, blueprint::Server, String> {
  fn from(_value: ServerResponseHeaderTransform) -> Self {
    Transform::new(move |config_server: &config::Server, mut server: blueprint::Server| {
      let headers = config_server
        .response_headers
        .0
        .clone()
        .validate_all(|(k, v)| {
          let name = HeaderName::from_bytes(k.as_bytes())
            .map_err(|e| ValidationError::new(format!("Parsing failed because of {}", e)));
          let value = HeaderValue::from_str(v.as_str())
            .map_err(|e| ValidationError::new(format!("Parsing failed because of {}", e)));
          name.validate_both(value)
        })
        .trace("responseHeaders")
        .trace("@server")
        .trace("schema")?;

      let mut response_headers = HeaderMap::new();
      response_headers.extend(headers);
      server = server.clone().response_headers(response_headers);
      Ok(server)
    })
  }
}

/// Handle the base url in the [`config::Server`]
pub struct ServerBaseUrlTransform;

impl From<ServerBaseUrlTransform> for Transform<config::Server, blueprint::Server, String> {
  fn from(_value: ServerBaseUrlTransform) -> Self {
    Transform::new(move |config_server: &config::Server, mut server: blueprint::Server| {
      if let Some(base_url) = config_server.upstream.base_url.clone() {
        Valid::Ok(reqwest::Url::parse(base_url.as_str()).map_err(|e| ValidationError::new(e.to_string()))?)?;
        server.upstream = server.clone().upstream.base_url(Some(base_url));
      }
      Ok(server)
    })
  }
}

/// Configer the outher server settings
pub struct ServerCmpletefTransform;

impl From<ServerCmpletefTransform> for Transform<config::Server, blueprint::Server, String> {
  fn from(_value: ServerCmpletefTransform) -> Self {
    Transform::new(move |config_server: &config::Server, mut server: blueprint::Server| {
      server = server
        .clone()
        .enable_apollo_tracing(config_server.enable_apollo_tracing.unwrap_or_default())
        .enable_cache_control_header(config_server.enable_cache_control_header.unwrap_or_default())
        .enable_introspection(config_server.enable_introspection.unwrap_or_default())
        .enable_query_validation(config_server.enable_query_validation.unwrap_or_default())
        .enable_response_validation(config_server.enable_response_validation.unwrap_or_default())
        .global_response_timeout(config_server.global_response_timeout.unwrap_or_default())
        .port(config_server.port.unwrap_or_default())
        .upstream(config_server.upstream.clone())
        .vars(config_server.vars.clone().0);

      server.upstream = server
        .upstream
        .clone()
        .allowed_headers(config_server.upstream.allowed_headers.clone())
        .batch(config_server.upstream.batch.clone())
        .enable_http_cache(config_server.upstream.enable_http_cache);

      Ok(server)
    })
  }
}
