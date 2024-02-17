use anyhow::Result;

use super::basic::BasicVerifier;
use super::error::Error;
use super::jwt::jwt_verify::JwtVerifier;
use crate::blueprint;
use crate::http::RequestContext;
use crate::runtime::TargetRuntime;

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
    pub fn new(config: blueprint::AuthProvider, runtime: &TargetRuntime) -> Self {
        match config {
            blueprint::AuthProvider::Basic(options) => {
                AuthVerifier::Basic(BasicVerifier::new(options))
            }
            blueprint::AuthProvider::Jwt(options) => {
                AuthVerifier::Jwt(JwtVerifier::new(options, runtime))
            }
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
