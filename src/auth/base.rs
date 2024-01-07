use std::sync::Arc;

use thiserror::Error;

use super::basic::BasicProvider;
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
  Basic(BasicProvider),
  Jwt(JwtProvider),
}

impl AuthProvider {
  pub fn from_config(config: blueprint::AuthProvider, client: Arc<dyn HttpClient>) -> Self {
    match config {
      blueprint::AuthProvider::Basic(options) => AuthProvider::Basic(BasicProvider::new(options)),
      blueprint::AuthProvider::Jwt(options) => AuthProvider::Jwt(JwtProvider::new(options, client)),
    }
  }
}

impl AuthProviderTrait for AuthProvider {
  async fn validate(&self, req_ctx: &RequestContext) -> Valid<(), AuthError> {
    match self {
      AuthProvider::Basic(basic) => basic.validate(req_ctx).await,
      AuthProvider::Jwt(jwt) => jwt.validate(req_ctx).await,
    }
  }
}
