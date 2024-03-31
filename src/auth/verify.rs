use std::cmp::max;

use anyhow::Result;
use futures_util::{join, TryFutureExt};

use super::basic::BasicVerifier;
use super::error::Error;
use super::jwt::jwt_verify::JwtVerifier;
use crate::blueprint;
use crate::http::RequestContext;

#[async_trait::async_trait]
pub(crate) trait Verify {
    async fn verify(&self, req_ctx: &RequestContext) -> Result<(), Error>;
}

pub enum Verifier {
    Basic(BasicVerifier),
    Jwt(JwtVerifier),
}

pub enum AuthVerifier {
    Single(Verifier),
    And(Box<AuthVerifier>, Box<AuthVerifier>),
    Or(Box<AuthVerifier>, Box<AuthVerifier>),
}

impl From<blueprint::AuthProvider> for Verifier {
    fn from(provider: blueprint::AuthProvider) -> Self {
        match provider {
            blueprint::AuthProvider::Basic(options) => Verifier::Basic(BasicVerifier::new(options)),
            blueprint::AuthProvider::Jwt(options) => Verifier::Jwt(JwtVerifier::new(options)),
        }
    }
}

impl From<blueprint::Auth> for AuthVerifier {
    fn from(provider: blueprint::Auth) -> Self {
        match provider {
            blueprint::Auth::Single(provider) => AuthVerifier::Single(provider.into()),
            blueprint::Auth::And(left, right) => {
                AuthVerifier::And(Box::new((*left).into()), Box::new((*right).into()))
            }
            blueprint::Auth::Or(left, right) => {
                AuthVerifier::Or(Box::new((*left).into()), Box::new((*right).into()))
            }
        }
    }
}

#[async_trait::async_trait]
impl Verify for Verifier {
    async fn verify(&self, req_ctx: &RequestContext) -> Result<(), Error> {
        match self {
            Verifier::Basic(basic) => basic.verify(req_ctx).await,
            Verifier::Jwt(jwt) => jwt.verify(req_ctx).await,
        }
    }
}

#[async_trait::async_trait]
impl Verify for AuthVerifier {
    async fn verify(&self, req_ctx: &RequestContext) -> Result<(), Error> {
        match self {
            AuthVerifier::Single(verifier) => verifier.verify(req_ctx).await,
            AuthVerifier::And(left, right) => {
                match join!(left.verify(req_ctx), right.verify(req_ctx)) {
                    (Ok(_), Ok(_)) => Ok(()),
                    (Ok(_), Err(err)) | (Err(err), Ok(_)) => Err(err),
                    (Err(e1), Err(e2)) => Err(max(e1, e2)),
                }
            }
            AuthVerifier::Or(left, right) => {
                left.verify(req_ctx)
                    .or_else(|e1| async {
                        if let Err(e2) = right.verify(req_ctx).await {
                            Err(max(e1, e2))
                        } else {
                            Ok(())
                        }
                    })
                    .await
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AuthVerifier;
    use crate::auth::basic::tests::create_basic_auth_request;
    use crate::auth::error::Error;
    use crate::auth::jwt::jwt_verify::tests::{create_jwt_auth_request, JWT_VALID_TOKEN_WITH_KID};
    use crate::auth::verify::Verify;
    use crate::blueprint::{Auth, AuthProvider, BasicProvider, JwtProvider};

    #[tokio::test]
    async fn verify() {
        let verifier = AuthVerifier::from(Auth::Single(AuthProvider::Basic(
            BasicProvider::test_value(),
        )));
        let req_ctx = create_basic_auth_request("testuser1", "wrong-password");

        assert_eq!(verifier.verify(&req_ctx).await, Err(Error::Invalid));

        let req_ctx = create_basic_auth_request("testuser1", "password123");

        assert_eq!(verifier.verify(&req_ctx).await, Ok(()));
    }

    #[tokio::test]
    async fn verify_and() {
        let verifier = AuthVerifier::from(Auth::And(
            Auth::Single(AuthProvider::Basic(BasicProvider::test_value())).into(),
            Auth::Single(AuthProvider::Basic(BasicProvider::test_value())).into(),
        ));
        let req_ctx = create_basic_auth_request("testuser1", "wrong-password");

        assert_eq!(verifier.verify(&req_ctx).await, Err(Error::Invalid));

        let req_ctx = create_basic_auth_request("testuser1", "password123");

        assert_eq!(verifier.verify(&req_ctx).await, Ok(()));
    }

    #[tokio::test]
    async fn verify_any() {
        let verifier = AuthVerifier::from(Auth::Or(
            Auth::Single(AuthProvider::Basic(BasicProvider::test_value())).into(),
            Auth::Single(AuthProvider::Jwt(JwtProvider::test_value())).into(),
        ));
        let req_ctx = create_basic_auth_request("testuser1", "wrong-password");

        assert_eq!(verifier.verify(&req_ctx).await, Err(Error::Invalid));

        let req_ctx = create_basic_auth_request("testuser1", "password123");

        assert_eq!(verifier.verify(&req_ctx).await, Ok(()));

        let req_ctx = create_jwt_auth_request(JWT_VALID_TOKEN_WITH_KID);

        assert_eq!(verifier.verify(&req_ctx).await, Ok(()));
    }
}
