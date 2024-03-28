use std::cmp::max;

use anyhow::Result;
use futures_util::future::join_all;

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
    Any(Vec<AuthVerifier>),
    All(Vec<AuthVerifier>),
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
            blueprint::Auth::All(providers) => {
                let verifiers = providers.into_iter().map(AuthVerifier::from).collect();

                AuthVerifier::All(verifiers)
            }
            blueprint::Auth::Any(providers) => {
                let verifiers = providers.into_iter().map(AuthVerifier::from).collect();

                AuthVerifier::Any(verifiers)
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
            AuthVerifier::All(verifiers) => {
                for verifier in verifiers {
                    verifier.verify(req_ctx).await?
                }

                Ok(())
            }
            AuthVerifier::Any(verifiers) => {
                let results =
                    join_all(verifiers.iter().map(|verifier| verifier.verify(req_ctx))).await;

                let mut error = Error::Missing;

                for result in results {
                    if let Err(err) = result {
                        error = max(error, err);
                    } else {
                        return Ok(());
                    }
                }

                Err(error)
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
    async fn verify_all() {
        let verifier = AuthVerifier::from(Auth::All(Vec::default()));
        let req_ctx = create_basic_auth_request("testuser1", "wrong-password");

        assert_eq!(verifier.verify(&req_ctx).await, Ok(()));

        let verifier = AuthVerifier::from(Auth::All(vec![Auth::Single(AuthProvider::Basic(
            BasicProvider::test_value(),
        ))]));

        assert_eq!(verifier.verify(&req_ctx).await, Err(Error::Invalid));

        let req_ctx = create_basic_auth_request("testuser1", "password123");

        assert_eq!(verifier.verify(&req_ctx).await, Ok(()));

        let verifier = AuthVerifier::from(Auth::All(vec![
            Auth::Single(AuthProvider::Basic(BasicProvider::test_value())),
            Auth::Single(AuthProvider::Jwt(JwtProvider::test_value())),
        ]));

        assert_eq!(verifier.verify(&req_ctx).await, Err(Error::Missing));
    }

    #[tokio::test]
    async fn verify_any() {
        let verifier = AuthVerifier::from(Auth::Any(Vec::default()));
        let req_ctx = create_basic_auth_request("testuser1", "wrong-password");

        assert_eq!(verifier.verify(&req_ctx).await, Err(Error::Missing));

        let verifier = AuthVerifier::from(Auth::Any(vec![Auth::Single(AuthProvider::Basic(
            BasicProvider::test_value(),
        ))]));

        assert_eq!(verifier.verify(&req_ctx).await, Err(Error::Invalid));

        let req_ctx = create_basic_auth_request("testuser1", "password123");

        assert_eq!(verifier.verify(&req_ctx).await, Ok(()));

        let verifier = AuthVerifier::from(Auth::Any(vec![
            Auth::Single(AuthProvider::Basic(BasicProvider::test_value())),
            Auth::Single(AuthProvider::Jwt(JwtProvider::test_value())),
        ]));

        assert_eq!(verifier.verify(&req_ctx).await, Ok(()));

        let req_ctx = create_jwt_auth_request(JWT_VALID_TOKEN_WITH_KID);

        assert_eq!(verifier.verify(&req_ctx).await, Ok(()));
    }
}
