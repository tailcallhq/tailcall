use std::sync::Arc;

use thiserror::Error;

use super::jwt::JwtProvider;
use crate::config::AuthProviderConfig;
use crate::http::{HttpClient, RequestContext};
use crate::valid::Valid;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum AuthError {
  #[error("Haven't found auth parameters")]
  Missing,
  #[error("Couldn't validate auth request")]
  ValidationNotAccessible,
  #[error("Auth validation failed")]
  ValidationFailed,
}

pub(crate) trait AuthProviderTrait {
  async fn validate(&self, req_ctx: &RequestContext) -> Valid<(), AuthError>;
}

pub enum AuthProvider {
  JWT(JwtProvider),
}

impl AuthProvider {
  pub fn from_config(config: AuthProviderConfig, client: Arc<dyn HttpClient>) -> Valid<Self, String> {
    match config {
      AuthProviderConfig::JWT(options) => JwtProvider::new(options, client.clone())
        .map(AuthProvider::JWT)
        .trace("JWT"),
    }
  }
}

impl AuthProviderTrait for AuthProvider {
  async fn validate(&self, req_ctx: &RequestContext) -> Valid<(), AuthError> {
    match self {
      AuthProvider::JWT(jwt) => jwt.validate(req_ctx).await,
    }
  }
}
