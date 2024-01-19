use std::sync::Arc;

use super::basic::BasicVerifier;
use super::error::Error;
use super::jwt::JwtVerifier;
use crate::http::RequestContext;
use crate::{blueprint, HttpIO};

pub(crate) trait Verify {
  async fn verify(&self, req_ctx: &RequestContext) -> Result<(), Error>;
}

#[allow(clippy::large_enum_variant)]
// The difference in size is indeed significant here
// but it's quite unlikely that someone will require to store several hundreds
// of providers or more to care much
pub enum AuthVerifier {
  Basic(BasicVerifier),
  Jwt(JwtVerifier),
}

impl AuthVerifier {
  pub fn from_config(config: blueprint::AuthProvider, client: Arc<dyn HttpIO>) -> Self {
    match config {
      blueprint::AuthProvider::Basic(options) => AuthVerifier::Basic(BasicVerifier::new(options)),
      blueprint::AuthProvider::Jwt(options) => AuthVerifier::Jwt(JwtVerifier::new(options, client)),
    }
  }
}

impl Verify for AuthVerifier {
  async fn verify(&self, req_ctx: &RequestContext) -> Result<(), Error> {
    match self {
      AuthVerifier::Basic(basic) => basic.verify(req_ctx).await,
      AuthVerifier::Jwt(jwt) => jwt.verify(req_ctx).await,
    }
  }
}
