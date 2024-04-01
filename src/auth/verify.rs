use futures_util::join;

use super::basic::BasicVerifier;
use super::jwt::jwt_verify::JwtVerifier;
use super::verification::Verification;
use crate::blueprint;
use crate::http::RequestContext;

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
    use crate::auth::basic::tests::create_basic_auth_request;
    use crate::auth::error::Error;
    use crate::auth::jwt::jwt_verify::tests::{create_jwt_auth_request, JWT_VALID_TOKEN_WITH_KID};
    use crate::auth::verification::Verification;
    use crate::auth::verify::Verify;
    use crate::blueprint::{Auth, Basic, Jwt, Provider};

    #[tokio::test]
    async fn verify() {
        let verifier = AuthVerifier::from(Auth::Provider(Provider::Basic(Basic::test_value())));
        let req_ctx = create_basic_auth_request("testuser1", "wrong-password");

        assert_eq!(
            verifier.verify(&req_ctx).await,
            Verification::fail(Error::Invalid)
        );

        let req_ctx = create_basic_auth_request("testuser1", "password123");

        assert_eq!(verifier.verify(&req_ctx).await, Verification::succeed());
    }

    #[tokio::test]
    async fn verify_and() {
        let verifier = AuthVerifier::from(Auth::And(
            Auth::Provider(Provider::Basic(Basic::test_value())).into(),
            Auth::Provider(Provider::Basic(Basic::test_value())).into(),
        ));
        let req_ctx = create_basic_auth_request("testuser1", "wrong-password");

        assert_eq!(
            verifier.verify(&req_ctx).await,
            Verification::fail(Error::Invalid)
        );

        let req_ctx = create_basic_auth_request("testuser1", "password123");

        assert_eq!(verifier.verify(&req_ctx).await, Verification::succeed());
    }

    #[tokio::test]
    async fn verify_any() {
        let verifier = AuthVerifier::from(Auth::Or(
            Auth::Provider(Provider::Basic(Basic::test_value())).into(),
            Auth::Provider(Provider::Jwt(Jwt::test_value())).into(),
        ));
        let req_ctx = create_basic_auth_request("testuser1", "wrong-password");

        assert_eq!(
            verifier.verify(&req_ctx).await,
            Verification::fail(Error::Invalid)
        );

        let req_ctx = create_basic_auth_request("testuser1", "password123");

        assert_eq!(verifier.verify(&req_ctx).await, Verification::succeed());

        let req_ctx = create_jwt_auth_request(JWT_VALID_TOKEN_WITH_KID);

        assert_eq!(verifier.verify(&req_ctx).await, Verification::succeed());
    }
}
