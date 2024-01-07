use headers::authorization::Basic;
use headers::{Authorization, HeaderMapExt};
use htpasswd_verify::Htpasswd;

use super::base::{AuthError, AuthProviderTrait};
use crate::blueprint;
use crate::http::RequestContext;
use crate::valid::Valid;

pub struct BasicProvider {
  verifier: Htpasswd<'static>,
}

impl AuthProviderTrait for BasicProvider {
  async fn validate(&self, req_ctx: &RequestContext) -> Valid<(), AuthError> {
    let header = req_ctx.req_headers.typed_get::<Authorization<Basic>>();

    let Some(header) = header else {
      return Valid::fail(AuthError::Missing);
    };

    if self.verifier.check(header.username(), header.password()) {
      Valid::succeed(())
    } else {
      Valid::fail(AuthError::ValidationFailed)
    }
  }
}

impl BasicProvider {
  pub fn new(options: blueprint::BasicProvider) -> Self {
    Self { verifier: Htpasswd::new_owned(&options.htpasswd) }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::valid::ValidationError;

  // testuser1:password123
  // testuser2:mypassword
  // testuser3:abc123
  pub static HTPASSWD_TEST: &str = "
testuser1:$apr1$e3dp9qh2$fFIfHU9bilvVZBl8TxKzL/
testuser2:$2y$10$wJ/mZDURcAOBIrswCAKFsO0Nk7BpHmWl/XuhF7lNm3gBAFH3ofsuu
testuser3:{SHA}Y2fEjdGT1W6nsLqtJbGUVeUp9e4=
";

  fn create_auth_request(username: &str, password: &str) -> RequestContext {
    let mut req_context = RequestContext::default();

    req_context
      .req_headers
      .typed_insert(Authorization::basic(username, password));

    req_context
  }

  #[tokio::test]
  async fn verify_passwords() {
    let provider = BasicProvider::new(blueprint::BasicProvider { htpasswd: HTPASSWD_TEST.to_owned() });

    let validation = provider.validate(&RequestContext::default()).await.to_result().err();
    assert_eq!(validation, Some(ValidationError::new(AuthError::Missing)));

    let validation = provider
      .validate(&create_auth_request("testuser1", "wrong-password"))
      .await
      .to_result()
      .err();
    assert_eq!(validation, Some(ValidationError::new(AuthError::ValidationFailed)));

    let validation = provider
      .validate(&create_auth_request("testuser1", "password123"))
      .await;
    assert!(validation.is_succeed());

    let validation = provider.validate(&create_auth_request("testuser2", "mypassword")).await;
    assert!(validation.is_succeed());

    let validation = provider.validate(&create_auth_request("testuser3", "abc123")).await;
    assert!(validation.is_succeed());
  }
}
