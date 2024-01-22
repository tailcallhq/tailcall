use anyhow::Result;

use super::basic::BasicVerifier;
use super::error::Error;
use super::jwt::jwt_verify::JwtVerifier;
use crate::http::RequestContext;
use crate::init_context::InitContext;
use crate::{blueprint, EnvIO, HttpIO};

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
  pub fn try_new<Http: HttpIO, Env: EnvIO>(
    config: blueprint::AuthProvider,
    init_context: &InitContext<Http, Env>,
  ) -> Result<Self> {
    match config {
      blueprint::AuthProvider::Basic(options) => {
        Ok(AuthVerifier::Basic(BasicVerifier::try_new(options, init_context)?))
      }
      blueprint::AuthProvider::Jwt(options) => Ok(AuthVerifier::Jwt(JwtVerifier::try_new(options, init_context)?)),
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
