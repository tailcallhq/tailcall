use headers::authorization::Basic;
use headers::{Authorization, HeaderMapExt};
use htpasswd_verify::Htpasswd;

use super::error::Error;
use super::verification::Verification;
use super::verify::Verify;
use crate::core::blueprint;
use crate::core::http::RequestContext;

pub struct BasicVerifier {
    verifier: Htpasswd<'static>,
}

#[async_trait::async_trait]
impl Verify for BasicVerifier {
    /// Verify the request context against the basic auth provider.
    async fn verify(&self, req_ctx: &RequestContext) -> Verification {
        let header = req_ctx
            .allowed_headers
            .typed_try_get::<Authorization<Basic>>();

        let Ok(header) = header else {
            return Verification::fail(Error::Invalid);
        };

        let Some(header) = header else {
            return Verification::fail(Error::Missing);
        };
        if self.verifier.check(header.username(), header.password()) {
            Verification::succeed()
        } else {
            Verification::fail(Error::Invalid)
        }
    }
}

impl BasicVerifier {
    pub fn new(options: blueprint::Basic) -> Self {
        Self { verifier: Htpasswd::new_owned(&options.htpasswd) }
    }
}

#[cfg(test)]
pub mod tests {
    use http::header::HeaderValue;

    use super::*;

    // testuser1:password123
    // testuser2:mypassword
    // testuser3:abc123
    pub static HTPASSWD_TEST: &str = "
testuser1:$apr1$e3dp9qh2$fFIfHU9bilvVZBl8TxKzL/
testuser2:$2y$10$wJ/mZDURcAOBIrswCAKFsO0Nk7BpHmWl/XuhF7lNm3gBAFH3ofsuu
testuser3:{SHA}Y2fEjdGT1W6nsLqtJbGUVeUp9e4=
";

    impl blueprint::Basic {
        pub fn test_value() -> Self {
            Self { htpasswd: HTPASSWD_TEST.to_owned() }
        }
    }

    pub fn create_basic_auth_request(username: &str, password: &str) -> RequestContext {
        let mut req_context = RequestContext::default();

        req_context
            .allowed_headers
            .typed_insert(Authorization::basic(username, password));

        req_context
    }

    #[tokio::test]
    async fn verify_missing_credentials() {
        let provider = setup_provider();
        let validation = provider.verify(&RequestContext::default()).await;
        assert_eq!(validation, Verification::fail(Error::Missing));
    }

    #[tokio::test]
    async fn verify_wrong_password() {
        let provider = setup_provider();
        let validation = provider
            .verify(&create_basic_auth_request("testuser1", "wrong-password"))
            .await;
        assert_eq!(validation, Verification::fail(Error::Invalid));
    }

    #[tokio::test]
    async fn verify_correct_password_testuser1() {
        let provider = setup_provider();
        let validation = provider
            .verify(&create_basic_auth_request("testuser1", "password123"))
            .await;
        assert_eq!(validation, Verification::succeed());
    }

    #[tokio::test]
    async fn verify_correct_password_testuser2() {
        let provider = setup_provider();
        let validation = provider
            .verify(&create_basic_auth_request("testuser2", "mypassword"))
            .await;
        assert_eq!(validation, Verification::succeed());
    }

    #[tokio::test]
    async fn verify_correct_password_testuser3() {
        let provider = setup_provider();
        let validation = provider
            .verify(&create_basic_auth_request("testuser3", "abc123"))
            .await;
        assert_eq!(validation, Verification::succeed());
    }

    #[tokio::test]
    async fn verify_auth_failure() {
        let provider = setup_provider();
        let mut req_ctx = RequestContext::default();
        req_ctx.allowed_headers.insert(
            "Authorization",
            HeaderValue::from_static("Basic dGVzdHVzZXIyOm15cGFzc3dvcmQ"),
        );
        let validation = provider.verify(&req_ctx).await;
        assert_eq!(validation, Verification::fail(Error::Invalid));
    }

    // Helper function for setting up the provider
    fn setup_provider() -> BasicVerifier {
        BasicVerifier::new(blueprint::Basic { htpasswd: HTPASSWD_TEST.to_owned() })
    }
}
