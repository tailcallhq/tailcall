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

#[allow(clippy::large_enum_variant)]
// The difference in size is indeed significant here
// but it's quite unlikely that someone will require to store several hundreds
// of providers or more to care much
pub enum AuthVerifier {
    Basic(BasicVerifier),
    Jwt(JwtVerifier),
    Or(Box<AuthVerifier>, Box<AuthVerifier>),
    And(Box<AuthVerifier>, Box<AuthVerifier>),
}

impl AuthVerifier {
    pub fn new(provider: blueprint::Auth) -> Option<Self> {
        match provider {
            blueprint::Auth::Basic(options) => {
                Some(AuthVerifier::Basic(BasicVerifier::new(options)))
            }
            blueprint::Auth::Jwt(options) => Some(AuthVerifier::Jwt(JwtVerifier::new(options))),
            blueprint::Auth::And(a, b) => {
                let a = Self::new(*a);
                let b = Self::new(*b);

                match (a, b) {
                    (None, None) => None,
                    (Some(a), None) => Some(a),
                    (None, Some(b)) => Some(b),
                    (Some(a), Some(b)) => Some(AuthVerifier::And(Box::new(a), Box::new(b))),
                }
            }
            blueprint::Auth::Or(a, b) => {
                let a = Self::new(*a);
                let b = Self::new(*b);

                match (a, b) {
                    (None, None) => None,
                    (Some(a), None) => Some(a),
                    (None, Some(b)) => Some(b),
                    (Some(a), Some(b)) => Some(AuthVerifier::Or(Box::new(a), Box::new(b))),
                }
            }
            blueprint::Auth::Empty => None,
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
            AuthVerifier::Basic(basic) => basic.verify(req_ctx).await,
            AuthVerifier::Jwt(jwt) => jwt.verify(req_ctx).await,
            AuthVerifier::Or(left, right) => {
                let left_result = left.verify(req_ctx).await;
                if left_result.is_err() {
                    right.verify(req_ctx).await
                } else {
                    Ok(())
                }
            }
            AuthVerifier::And(left, right) => {
                let (a, b) = left.verify(req_ctx).join(right.verify(req_ctx)).await;
                match (a, b) {
                    (Ok(_), Ok(_)) => Ok(()),
                    (Ok(_), Err(e)) => Err(e),
                    (Err(e), Ok(_)) => Err(e),
                    (Err(e1), Err(e2)) => Err(e1.max(e2)),
                }
            }
        }
    }
}
