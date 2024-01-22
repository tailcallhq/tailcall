use std::sync::{Arc, Mutex};

use anyhow::Result;
use futures_util::future::join_all;

use super::error::Error;
use super::verify::{AuthVerifier, Verify};
use crate::blueprint::Auth;
use crate::http::RequestContext;
use crate::init_context::InitContext;
use crate::{EnvIO, HttpIO};

#[derive(Default)]
pub struct GlobalAuthContext {
  providers: Vec<AuthVerifier>,
}

#[derive(Default)]
pub struct AuthContext {
  // TODO: can we do without mutex?
  auth_result: Mutex<Option<Result<(), Error>>>,
  global_ctx: Arc<GlobalAuthContext>,
}

impl GlobalAuthContext {
  // TODO: it could be better to return async_graphql::Error to make it more graphql way
  // with additional info. But this actually requires rewrites to expression to work with that type
  // since otherwise any additional info will be lost during conversion to anyhow::Error
  async fn validate(&self, request: &RequestContext) -> Result<(), Error> {
    let validations = join_all(self.providers.iter().map(|provider| provider.verify(request))).await;

    let mut error = Error::Missing;

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
  pub fn try_new<Http: HttpIO, Env: EnvIO>(auth: Auth, init_context: &InitContext<Http, Env>) -> Result<Self> {
    let providers = auth
      .0
      .into_iter()
      .map(|provider| AuthVerifier::try_new(provider.provider, init_context))
      .collect::<Result<_>>()?;

    Ok(Self { providers })
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
    Self { global_ctx: global_ctx.clone(), auth_result: Default::default() }
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
  use crate::http::Response;
  use crate::mustache::Mustache;

  struct MockHttpClient;

  #[async_trait::async_trait]
  impl HttpIO for MockHttpClient {
    async fn execute(&self, _request: reqwest::Request) -> anyhow::Result<Response<hyper::body::Bytes>> {
      todo!()
    }
  }

  struct MockEnv;

  impl EnvIO for MockEnv {
    fn get(&self, _key: &str) -> Option<std::borrow::Cow<'_, str>> {
      todo!()
    }
  }

  impl InitContext<MockHttpClient, MockEnv> {
    fn test_value() -> Self {
      InitContext { vars: Default::default(), env: Arc::new(MockEnv), http_client: Arc::new(MockHttpClient) }
    }
  }

  #[tokio::test]
  async fn validate_request() -> Result<()> {
    let init_context = InitContext::test_value();
    let basic_provider = BasicVerifier::try_new(
      blueprint::BasicProvider { htpasswd: Mustache::parse(HTPASSWD_TEST)? },
      &init_context,
    )?;
    let jwt_options = blueprint::JwtProvider::test_value();
    let jwt_provider = JwtVerifier::try_new(jwt_options, &init_context)?;

    let auth_context =
      GlobalAuthContext { providers: vec![AuthVerifier::Basic(basic_provider), AuthVerifier::Jwt(jwt_provider)] };

    let validation = auth_context.validate(&RequestContext::default()).await.err();
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
