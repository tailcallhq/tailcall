use std::sync::{Arc, Mutex};

use anyhow::Result;

use super::error::Error;
use super::verify::{AuthVerifier, Verify};
use crate::blueprint::Auth;
use crate::http::RequestContext;

#[derive(Default)]
pub struct GlobalAuthContext {
    verifier: AuthVerifier,
}

#[derive(Default)]
pub struct AuthContext {
    // TODO: can we do without mutex?
    auth_result: Mutex<Option<Result<(), Error>>>,
    global_ctx: Arc<GlobalAuthContext>,
}

impl GlobalAuthContext {
    // TODO: it could be better to return async_graphql::Error to make it more
    // graphql way with additional info. But this actually requires rewrites to
    // expression to work with that type since otherwise any additional info
    // will be lost during conversion to anyhow::Error
    async fn validate(&self, request: &RequestContext) -> Result<(), Error> {
        self.verifier.verify(request).await
    }
}

impl GlobalAuthContext {
    pub fn new(auth: Auth) -> Self {
        let verifier = AuthVerifier::new(auth);
        Self { verifier }
    }
}

impl AuthContext {
    pub async fn validate(&self, request: &RequestContext) -> Result<(), Error> {
        if let Some(result) = self.auth_result.lock().unwrap().as_ref() {
            return result.clone();
        }

        let result = self.global_ctx.validate(request).await;

        self.auth_result.lock().unwrap().replace(result.clone());

        result
    }
}

impl From<&Arc<GlobalAuthContext>> for AuthContext {
    fn from(global_ctx: &Arc<GlobalAuthContext>) -> Self {
        Self {
            global_ctx: global_ctx.clone(),
            auth_result: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::basic::tests::{create_basic_auth_request, HTPASSWD_TEST};
    use crate::auth::basic::BasicVerifier;
    use crate::auth::jwt::jwt_verify::tests::{create_jwt_auth_request, JWT_VALID_TOKEN_WITH_KID};
    use crate::auth::jwt::jwt_verify::JwtVerifier;
    use crate::blueprint;

    #[tokio::test]
    async fn validate_request() -> Result<()> {
        let basic_provider =
            BasicVerifier::new(blueprint::BasicProvider { htpasswd: HTPASSWD_TEST.to_owned() });
        let jwt_options = blueprint::JwtProvider::test_value();
        let jwt_provider = JwtVerifier::new(jwt_options);

        let auth_context = GlobalAuthContext {
            verifier: AuthVerifier::Basic(basic_provider).or(AuthVerifier::Jwt(jwt_provider)),
        };

        let validation = auth_context
            .validate(&RequestContext::default())
            .await
            .err();
        assert_eq!(validation, Some(Error::Missing));

        let validation = auth_context
            .validate(&create_basic_auth_request("testuser1", "wrong-password"))
            .await
            .err();
        assert_eq!(validation, Some(Error::Invalid));

        let validation = auth_context
            .validate(&create_basic_auth_request("testuser1", "password123"))
            .await;
        assert!(validation.is_ok());

        let validation = auth_context
            .validate(&create_jwt_auth_request(JWT_VALID_TOKEN_WITH_KID))
            .await;
        assert!(validation.is_ok());

        Ok(())
    }
}
