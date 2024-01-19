use std::sync::Arc;

use thiserror::Error;

use super::basic::BasicVerifier;
use super::jwt::JWTVerifier;
use crate::http::RequestContext;
use crate::{blueprint, HttpIO};

#[derive(Debug, Error, Clone, PartialEq, PartialOrd)]
pub enum AuthError {
  #[error("Haven't found auth parameters")]
  Missing,
  #[error("Couldn't validate auth request")]
  // in case we haven't managed to actually validate the request
  // and have failed somewhere else, usually while executing request
  ValidationCheckFailed,
  #[error("Auth validation failed")]
  Invalid,
}

pub(crate) trait AuthVerifierTrait {
  async fn validate(&self, req_ctx: &RequestContext) -> Result<(), AuthError>;
}

#[allow(clippy::large_enum_variant)]
// the difference in size is indeed significant here
// but it's quite unlikely that someone will require to store several hundreds
// of providers or more to care much
pub enum AuthVerifier {
  Basic(BasicVerifier),
  Jwt(JWTVerifier),
}

impl AuthVerifier {
  pub fn from_config(config: blueprint::AuthProvider, client: Arc<dyn HttpIO>) -> Self {
    match config {
      blueprint::AuthProvider::Basic(options) => AuthVerifier::Basic(BasicVerifier::new(options)),
      blueprint::AuthProvider::Jwt(options) => AuthVerifier::Jwt(JWTVerifier::new(options, client)),
    }
  }
}

impl AuthVerifierTrait for AuthVerifier {
  async fn validate(&self, req_ctx: &RequestContext) -> Result<(), AuthError> {
    match self {
      AuthVerifier::Basic(basic) => basic.validate(req_ctx).await,
      AuthVerifier::Jwt(jwt) => jwt.validate(req_ctx).await,
    }
  }
}
