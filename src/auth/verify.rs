use anyhow::Result;
use async_std::prelude::FutureExt;

use super::basic::BasicVerifier;
use super::error::Error;
use super::jwt::jwt_verify::JwtVerifier;
use crate::blueprint;
use crate::http::RequestContext;

#[async_trait::async_trait]
pub(crate) trait Verify {
    async fn verify(&self, req_ctx: &RequestContext) -> Result<(), Error>;
}

#[derive(Default)]
#[allow(clippy::large_enum_variant)]
// The difference in size is indeed significant here
// but it's quite unlikely that someone will require to store several hundreds
// of providers or more to care much
pub enum AuthVerifier {
    Basic(BasicVerifier),
    Jwt(JwtVerifier),
    Or(Box<AuthVerifier>, Box<AuthVerifier>),
    And(Box<AuthVerifier>, Box<AuthVerifier>),
    #[default]
    Empty,
}

impl AuthVerifier {
    pub fn new(provider: blueprint::Auth) -> Self {
        match provider {
            blueprint::Auth::Basic(options) => AuthVerifier::Basic(BasicVerifier::new(options)),
            blueprint::Auth::Jwt(options) => AuthVerifier::Jwt(JwtVerifier::new(options)),
            blueprint::Auth::And(a, b) => {
                AuthVerifier::And(Box::new(Self::new(*a)), Box::new(Self::new(*b)))
            }
            blueprint::Auth::Or(a, b) => {
                AuthVerifier::Or(Box::new(Self::new(*a)), Box::new(Self::new(*b)))
            }
            blueprint::Auth::Empty => AuthVerifier::Empty,
        }
    }

    #[cfg(test)]
    pub fn or(self, other: AuthVerifier) -> Self {
        AuthVerifier::Or(Box::new(self), Box::new(other))
    }
}

#[async_trait::async_trait]
impl Verify for AuthVerifier {
    async fn verify(&self, req_ctx: &RequestContext) -> Result<(), Error> {
        match self {
            AuthVerifier::Empty => Ok(()),
            AuthVerifier::Basic(basic) => basic.verify(req_ctx).await,
            AuthVerifier::Jwt(jwt) => jwt.verify(req_ctx).await,
            AuthVerifier::Or(left, right) => {
                let left_result = left.verify(req_ctx).await;
                let right_result = right.verify(req_ctx).await;

                left_result.or(right_result)
            }
            AuthVerifier::And(left, right) => {
                let (a, b) = left.verify(req_ctx).join(right.verify(req_ctx)).await;

                match (a, b) {
                    (Ok(_), Ok(_)) => Ok(()),
                    (Ok(_), Err(e)) => Err(e),
                    (Err(e), Ok(_)) => Err(e),
                    (Err(e1), Err(e2)) => Err(e1.min(e2)),
                }
            }
        }
    }
}
