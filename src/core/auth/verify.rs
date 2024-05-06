use futures_util::join;

use super::basic::BasicVerifier;
use super::jwt::jwt_verify::JwtVerifier;
use super::verification::Verification;
use crate::core::blueprint;
use crate::core::http::RequestContext;

#[async_trait::async_trait]
pub(crate) trait Verify {
    async fn verify(&self, req_ctx: &RequestContext) -> Verification;
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

impl From<blueprint::Provider> for Verifier {
    fn from(provider: blueprint::Provider) -> Self {
        match provider {
            blueprint::Provider::Basic(options) => Verifier::Basic(BasicVerifier::new(options)),
            blueprint::Provider::Jwt(options) => Verifier::Jwt(JwtVerifier::new(options)),
        }
    }
}

impl From<blueprint::Auth> for AuthVerifier {
    fn from(provider: blueprint::Auth) -> Self {
        match provider {
            blueprint::Auth::Provider(provider) => AuthVerifier::Single(provider.into()),
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
    async fn verify(&self, req_ctx: &RequestContext) -> Verification {
        match self {
            Verifier::Basic(basic) => basic.verify(req_ctx).await,
            Verifier::Jwt(jwt) => jwt.verify(req_ctx).await,
        }
    }
}

#[async_trait::async_trait]
impl Verify for AuthVerifier {
    async fn verify(&self, req_ctx: &RequestContext) -> Verification {
        match self {
            AuthVerifier::Single(verifier) => verifier.verify(req_ctx).await,
            AuthVerifier::And(left, right) => {
                let (a, b) = join!(left.verify(req_ctx), right.verify(req_ctx));
                a.and(b)
            }
            AuthVerifier::Or(left, right) => {
                left.verify(req_ctx).await.or(right.verify(req_ctx).await)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AuthVerifier;
    use crate::core::auth::basic::tests::create_basic_auth_request;
    use crate::core::auth::error::Error;
    use crate::core::auth::jwt::jwt_verify::tests::{
        create_jwt_auth_request, JWT_VALID_TOKEN_WITH_KID,
    };
    use crate::core::auth::verification::Verification;
    use crate::core::auth::verify::Verify;
    use crate::core::blueprint::{Auth, Basic, Jwt, Provider};
    use crate::core::http::RequestContext;

    #[tokio::test]
    async fn verify_wrong_password() {
        let verifier = setup_basic_verifier();
        let req_ctx = create_basic_auth_request("testuser1", "wrong-password");
        verify_and_assert(&verifier, &req_ctx, Verification::fail(Error::Invalid)).await;
    }

    #[tokio::test]
    async fn verify_correct_password() {
        let verifier = setup_basic_verifier();
        let req_ctx = create_basic_auth_request("testuser1", "password123");
        verify_and_assert(&verifier, &req_ctx, Verification::succeed()).await;
    }

    #[tokio::test]
    async fn verify_and_wrong_password() {
        let verifier = setup_and_verifier();
        let req_ctx = create_basic_auth_request("testuser1", "wrong-password");
        verify_and_assert(&verifier, &req_ctx, Verification::fail(Error::Invalid)).await;
    }

    #[tokio::test]
    async fn verify_and_correct_password() {
        let verifier = setup_and_verifier();
        let req_ctx = create_basic_auth_request("testuser1", "password123");
        verify_and_assert(&verifier, &req_ctx, Verification::succeed()).await;
    }

    #[tokio::test]
    async fn verify_any_wrong_password() {
        let verifier = setup_or_verifier();
        let req_ctx = create_basic_auth_request("testuser1", "wrong-password");
        verify_and_assert(&verifier, &req_ctx, Verification::fail(Error::Invalid)).await;
    }

    #[tokio::test]
    async fn verify_any_correct_password() {
        let verifier = setup_or_verifier();
        let req_ctx = create_basic_auth_request("testuser1", "password123");
        verify_and_assert(&verifier, &req_ctx, Verification::succeed()).await;
    }

    #[tokio::test]
    async fn verify_any_jwt_valid_token() {
        let verifier = setup_or_verifier();
        let req_ctx = create_jwt_auth_request(JWT_VALID_TOKEN_WITH_KID);
        verify_and_assert(&verifier, &req_ctx, Verification::succeed()).await;
    }

    // Helper Functions
    async fn verify_and_assert(
        verifier: &AuthVerifier,
        req_ctx: &RequestContext,
        expected: Verification,
    ) {
        assert_eq!(verifier.verify(req_ctx).await, expected);
    }

    fn setup_basic_verifier() -> AuthVerifier {
        AuthVerifier::from(Auth::Provider(Provider::Basic(Basic::test_value())))
    }

    fn setup_and_verifier() -> AuthVerifier {
        AuthVerifier::from(Auth::And(
            Auth::Provider(Provider::Basic(Basic::test_value())).into(),
            Auth::Provider(Provider::Basic(Basic::test_value())).into(),
        ))
    }

    fn setup_or_verifier() -> AuthVerifier {
        AuthVerifier::from(Auth::Or(
            Auth::Provider(Provider::Basic(Basic::test_value())).into(),
            Auth::Provider(Provider::Jwt(Jwt::test_value())).into(),
        ))
    }
}
