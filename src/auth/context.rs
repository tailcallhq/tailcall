use std::sync::{Arc, Mutex};

use futures_util::future::join_all;

use super::base::{AuthError, AuthProvider, AuthProviderTrait};
use crate::config::Auth;
use crate::http::{HttpClient, RequestContext};
use crate::valid::Valid;

#[derive(Default)]
pub struct GlobalAuthContext {
  providers: Vec<AuthProvider>,
}

#[derive(Default)]
pub struct AuthContext {
  // TODO: can we do without mutex?
  auth_result: Mutex<Option<Valid<(), AuthError>>>,
  global_ctx: Arc<GlobalAuthContext>,
}

impl GlobalAuthContext {
  async fn validate(&self, request: &RequestContext) -> Valid<(), AuthError> {
    let validations = join_all(self.providers.iter().map(|provider| provider.validate(request))).await;

    Valid::from_iter(validations.into_iter(), |validation| validation).unit()
  }
}

impl GlobalAuthContext {
  pub fn new(auth: &Auth, client: Arc<dyn HttpClient>) -> Valid<Self, String> {
    Valid::from_iter(&auth.0, |provider| {
      AuthProvider::from_config(provider.provider.clone(), client.clone())
    })
    .map(|providers| Self { providers })
  }
}

impl AuthContext {
  pub async fn validate(&self, request: &RequestContext) -> Valid<(), AuthError> {
    if let Some(valid) = self.auth_result.lock().unwrap().as_ref() {
      return valid.clone();
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
