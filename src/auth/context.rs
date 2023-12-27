use std::sync::{Arc, Mutex};

use crate::{http::RequestContext, valid::Valid};

use super::{
  base::{AuthError, AuthProvider},
  jwt::JwtProvider,
};

#[derive(Default)]
pub struct GlobalAuthContext {
  // TODO: remove pub and create it from directive
  pub jwt_provider: Option<JwtProvider>,
}

#[derive(Default)]
pub struct AuthContext {
  // TODO: can we do without mutex?
  auth_result: Mutex<Option<Valid<(), AuthError>>>,
  global_ctx: Arc<GlobalAuthContext>,
}

impl GlobalAuthContext {
  async fn validate(&self, request: &RequestContext) -> Valid<(), AuthError> {
    if let Some(jwt_provider) = &self.jwt_provider {
      return jwt_provider.validate(request).await;
    }

    Valid::succeed(())
  }
}

impl AuthContext {
  pub async fn validate(&self, request: &RequestContext) -> Valid<(), AuthError> {
    if let Some(valid) = self.auth_result.lock().unwrap().as_ref() {
      dbg!("From cache", valid);
      return valid.clone();
    }

    let result = self.global_ctx.validate(request).await;

    dbg!("resolved", &result);

    self.auth_result.lock().unwrap().replace(result.clone());

    result
  }
}

impl From<&Arc<GlobalAuthContext>> for AuthContext {
  fn from(global_ctx: &Arc<GlobalAuthContext>) -> Self {
    Self { global_ctx: global_ctx.clone(), auth_result: Default::default() }
  }
}
