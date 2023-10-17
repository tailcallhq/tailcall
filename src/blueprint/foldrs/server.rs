use hyper::header::{HeaderName, HeaderValue};
use hyper::HeaderMap;

use crate::blueprint::{blueprint, Blueprint};
use crate::config::{self, Config};
use crate::try_fold::{TryFold, TryFolding};
use crate::valid::{Valid, ValidExtensions, ValidationError, VectorExtension};

const RESTRICTED_ROUTES: &[&str] = &["/", "/graphql"];

pub struct ServerFold;

impl TryFolding for ServerFold {
  type Input = Config;
  type Value = Blueprint;
  type Error = String;

  fn try_fold(self, cfg: &Self::Input, mut blueprint: Self::Value) -> Valid<Self::Value, Self::Error> {
    blueprint.server = TryFold::try_all(vec![
      ServerGraphqlFold,
      ServerResponseHeaderFold,
      ServerBaseUrlFold,
      ServerCompleteFold,
    ])
    .try_fold(&cfg.server, blueprint.server)?;
    Ok(blueprint)
  }
}

/// Handle the graphql in the [`config::Server`]
struct ServerGraphqlFold;

impl TryFolding for ServerGraphqlFold {
  type Input = config::Server;
  type Value = blueprint::Server;
  type Error = String;

  fn try_fold(self, server_cfg: &Self::Input, mut server: Self::Value) -> Valid<Self::Value, Self::Error> {
    if let Some(enable_graphiql) = server_cfg.enable_graphiql.clone() {
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
  }
}

/// Handle the responce headers in [`config::Server`]
struct ServerResponseHeaderFold;

impl TryFolding for ServerResponseHeaderFold {
  type Input = config::Server;
  type Value = blueprint::Server;
  type Error = String;

  fn try_fold(self, server_cfg: &Self::Input, mut server: Self::Value) -> Valid<Self::Value, Self::Error> {
    let headers = server_cfg
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
    Ok(server.clone().response_headers(response_headers))
  }
}
/// Handle the base url in the [`config::Server`]
pub struct ServerBaseUrlFold;

impl TryFolding for ServerBaseUrlFold {
  type Input = config::Server;
  type Value = blueprint::Server;
  type Error = String;

  fn try_fold(self, server_cfg: &Self::Input, mut server: Self::Value) -> Valid<Self::Value, Self::Error> {
    if let Some(base_url) = server_cfg.upstream.base_url.clone() {
      Valid::Ok(reqwest::Url::parse(base_url.as_str()).map_err(|e| ValidationError::new(e.to_string()))?)?;
      server.upstream = server.clone().upstream.base_url(Some(base_url));
    }
    Ok(server)
  }
}

/// Configer the outher server settings
struct ServerCompleteFold;

impl TryFolding for ServerCompleteFold {
  type Input = config::Server;
  type Value = blueprint::Server;
  type Error = String;

  fn try_fold(self, server_cfg: &Self::Input, mut server: Self::Value) -> Valid<Self::Value, Self::Error> {
    server = server
      .clone()
      .enable_apollo_tracing(server_cfg.enable_apollo_tracing.unwrap_or_default())
      .enable_cache_control_header(server_cfg.enable_cache_control_header.unwrap_or_default())
      .enable_introspection(server_cfg.enable_introspection.unwrap_or_default())
      .enable_query_validation(server_cfg.enable_query_validation.unwrap_or_default())
      .enable_response_validation(server_cfg.enable_response_validation.unwrap_or_default())
      .global_response_timeout(server_cfg.global_response_timeout.unwrap_or_default())
      .port(server_cfg.port.unwrap_or_default())
      .upstream(server_cfg.upstream.clone())
      .vars(server_cfg.vars.clone().0);

    server.upstream = server
      .upstream
      .clone()
      .allowed_headers(server_cfg.upstream.allowed_headers.clone())
      .batch(server_cfg.upstream.batch.clone())
      .enable_http_cache(server_cfg.upstream.enable_http_cache);

    Ok(server)
  }
}
