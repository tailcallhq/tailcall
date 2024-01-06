use std::sync::Arc;

use thiserror::Error;

use super::jwt::JwtProvider;
use crate::blueprint;
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
  pub fn from_config(config: blueprint::AuthProvider, client: Arc<dyn HttpClient>) -> Self {
    match config {
      blueprint::AuthProvider::JWT(options) => AuthProvider::JWT(JwtProvider::new(options, client)),
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
