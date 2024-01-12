use std::sync::{Arc, Mutex};

use futures_util::future::join_all;

use super::base::{AuthError, AuthProvider, AuthProviderTrait};
use crate::blueprint::Auth;
use crate::http::RequestContext;
use crate::io::HttpIO;

#[derive(Default)]
pub struct GlobalAuthContext {
  providers: Vec<AuthProvider>,
}

#[derive(Default)]
pub struct AuthContext {
  // TODO: can we do without mutex?
  auth_result: Mutex<Option<Result<(), AuthError>>>,
  global_ctx: Arc<GlobalAuthContext>,
}

impl GlobalAuthContext {
  // TODO: it could be better to return async_graphql::Error to make it more graphql way
  // with additional info. But this actually requires rewrites to expression to work with that type
  // since otherwise any additional info will be lost during conversion to anyhow::Error
  async fn validate(&self, request: &RequestContext) -> Result<(), AuthError> {
    let validations = join_all(self.providers.iter().map(|provider| provider.validate(request))).await;

    let mut error = AuthError::Missing;

    for v in validations {
      let Err(err) = v else {
        return Ok(());
      };

      if err > error {
        error = err;
      }
    }

    Err(error)
  }
}

impl GlobalAuthContext {
  pub fn new(auth: Auth, client: Arc<dyn HttpIO>) -> Self {
    let providers = auth
      .0
      .into_iter()
      .map(|provider| AuthProvider::from_config(provider.provider, client.clone()))
      .collect();

    Self { providers }
  }
}

impl AuthContext {
  pub async fn validate(&self, request: &RequestContext) -> Result<(), AuthError> {
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
    Self { global_ctx: global_ctx.clone(), auth_result: Default::default() }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::auth::basic::tests::{create_basic_auth_request, HTPASSWD_TEST};
  use crate::auth::basic::BasicProvider;
  use crate::auth::jwt::tests::{create_jwt_auth_request, JWT_VALID_TOKEN};
  use crate::auth::jwt::JwtProvider;
  use crate::blueprint;
  use crate::http::Response;

  struct MockHttpClient;

  #[async_trait::async_trait]
  impl HttpIO for MockHttpClient {
    async fn execute(&self, _req: reqwest::Request) -> anyhow::Result<Response<async_graphql::Value>> {
      todo!()
    }

    async fn execute_raw(&self, _req: reqwest::Request) -> anyhow::Result<Response<Vec<u8>>> {
      todo!()
    }
  }

  #[tokio::test]
  async fn validate_request() {
    let basic_provider = BasicProvider::new(blueprint::BasicProvider { htpasswd: HTPASSWD_TEST.to_owned() });
    let jwt_options = blueprint::JwtProvider::test_value();
    let jwt_provider = JwtProvider::new(jwt_options, Arc::new(MockHttpClient));

    let auth_context =
      GlobalAuthContext { providers: vec![AuthProvider::Basic(basic_provider), AuthProvider::Jwt(jwt_provider)] };

    let validation = auth_context.validate(&RequestContext::default()).await.err();
    assert_eq!(validation, Some(AuthError::Missing));

    let validation = auth_context
      .validate(&create_basic_auth_request("testuser1", "wrong-password"))
      .await
      .err();
    assert_eq!(validation, Some(AuthError::Invalid));

    let validation = auth_context
      .validate(&create_basic_auth_request("testuser1", "password123"))
      .await;
    assert!(validation.is_ok());

    let validation = auth_context.validate(&create_jwt_auth_request(JWT_VALID_TOKEN)).await;
    assert!(validation.is_ok());
  }
}
