mod remote_jwks;
mod validation;

use std::sync::Arc;

use headers::authorization::Bearer;
use headers::{Authorization, HeaderMapExt};
use jwtk::jwk::JwkSetVerifier;
use jwtk::HeaderAndClaims;

use self::remote_jwks::RemoteJwksVerifier;
use self::validation::{validate_aud, validate_iss};
use super::base::{AuthError, AuthProviderTrait};
use crate::blueprint;
use crate::http::{HttpClient, RequestContext};

// only used in tests and uses mocked implementation
#[cfg(test)]
impl blueprint::JwtProvider {
  pub fn test_value() -> Self {
    use std::fs;
    use std::path::PathBuf;

    use jwtk::jwk::JwkSet;

    let root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut test_file = root_dir.join(file!());

    test_file.pop();
    test_file.push("tests");
    test_file.push("jwks.json");

    let jwks = fs::read_to_string(test_file).unwrap();
    let jwks: JwkSet = serde_json::from_str(&jwks).unwrap();
    let jwks = blueprint::Jwks::Local(jwks);

    Self { issuer: Default::default(), audiences: Default::default(), optional_kid: false, jwks }
  }
}

enum JwksVerifier {
  Local(JwkSetVerifier),
  Remote(RemoteJwksVerifier),
}

impl JwksVerifier {
  pub fn new(options: &blueprint::JwtProvider, client: Arc<dyn HttpClient>) -> Self {
    match &options.jwks {
      blueprint::Jwks::Local(jwks) => {
        let mut verifier = jwks.verifier();

        verifier.set_require_kid(!options.optional_kid);

        Self::Local(verifier)
      }
      blueprint::Jwks::Remote { url, max_age } => {
        let mut verifier = RemoteJwksVerifier::new(url.clone(), client, *max_age);

        verifier.set_require_kid(!options.optional_kid);

        Self::Remote(verifier)
      }
    }
  }

  async fn verify(&self, token: &str) -> Result<HeaderAndClaims<()>, AuthError> {
    match self {
      JwksVerifier::Local(verifier) => verifier.verify(token).map_err(|_| AuthError::Invalid),
      JwksVerifier::Remote(verifier) => verifier.verify(token).await,
    }
  }
}

pub struct JwtProvider {
  options: blueprint::JwtProvider,
  verifier: JwksVerifier,
}

impl JwtProvider {
  pub fn new(options: blueprint::JwtProvider, client: Arc<dyn HttpClient>) -> Self {
    Self { verifier: JwksVerifier::new(&options, client), options }
  }

  fn resolve_token(&self, request: &RequestContext) -> Option<String> {
    let value = request.req_headers.typed_get::<Authorization<Bearer>>();

    value.map(|token| token.token().to_owned())
  }

  async fn validate_token(&self, token: &str) -> Result<(), AuthError> {
    let verification = self
      .verifier
      .verify(token)
      .await
      .map_err(|_| AuthError::ValidationCheckFailed)?;

    self.validate_claims(&verification)
  }

  fn validate_claims(&self, parsed: &HeaderAndClaims<()>) -> Result<(), AuthError> {
    let claims = parsed.claims();

    if !validate_iss(&self.options, claims) || !validate_aud(&self.options, claims) {
      return Err(AuthError::Invalid);
    }

    Ok(())
  }
}

impl AuthProviderTrait for JwtProvider {
  async fn validate(&self, request: &RequestContext) -> Result<(), AuthError> {
    let token = self.resolve_token(request);

    let Some(token) = token else {
      return Err(AuthError::Missing);
    };

    self.validate_token(&token).await
  }
}

#[cfg(test)]
pub mod tests {
  use std::collections::HashSet;

  use super::*;
  use crate::http::HttpClient;

  struct MockHttpClient;

  #[async_trait::async_trait]
  impl HttpClient for MockHttpClient {
    async fn execute(&self, _req: reqwest::Request) -> anyhow::Result<crate::http::Response> {
      todo!()
    }

    async fn execute_raw(&self, _req: reqwest::Request) -> anyhow::Result<reqwest::Response> {
      todo!()
    }
  }

  // token with issuer = "me" and audience = ["them"]
  // token is valid for 10 years. It it expired, update it =)
  // to parse the token and see its content use https://jwt.io
  pub const JWT_VALID_TOKEN: &str = "eyJhbGciOiJSUzI1NiIsImtpZCI6Ikk0OHFNSnA1NjZTU0tRb2dZWFl0SEJvOXE2WmNFS0hpeE5QZU5veFYxYzgifQ.eyJleHAiOjIwMTkwNTY0NDEuMCwiaXNzIjoibWUiLCJzdWIiOiJ5b3UiLCJhdWQiOlsidGhlbSJdfQ.cU-hJgVGWxK3-IBggYBChhf3FzibBKjuDLtq2urJ99FVXIGZls0VMXjyNW7yHhLLuif_9t2N5UIUIq-hwXVv7rrGRPCGrlqKU0jsUH251Spy7_ppG5_B2LsG3cBJcwkD4AVz8qjT3AaE_vYZ4WnH-CQ-F5Vm7wiYZgbdyU8xgKoH85KAxaCdJJlYOi8mApE9_zcdmTNJrTNd9sp7PX3lXSUu9AWlrZkyO-HhVbXFunVtfduDuTeVXxP8iw1wt6171CFbPmQJU_b3xCornzyFKmhSc36yvlDfoPPclWmWeyOfFEp9lVhQm0WhfDK7GiuRtaOxD-tOvpTjpcoZBeJb7bSg2OsneyeM_33a0WoPmjHw8WIxbroJz_PrfE72_TzbcTSDttKAv_e75PE48Vvx0661miFv4Gq8RBzMl2G3pQMEVCOm83v7BpodfN_YVJcqZJjVHMA70TZQ4K3L4_i9sIK9jJFfwEDVM7nsDnUu96n4vKs1fVvAuieCIPAJrfNOUMy7TwLvhnhUARsKnzmtNNrJuDhhBx-X93AHcG3micXgnqkFdKn6-ZUZ63I2KEdmjwKmLTRrv4n4eZKrRN-OrHPI4gLxJUhmyPAHzZrikMVBcDYfALqyki5SeKkwd4v0JAm87QzR4YwMdKErr0Xa5JrZqHGe2TZgVO4hIc-KrPw";

  pub fn create_jwt_auth_request(token: &str) -> RequestContext {
    let mut req_context = RequestContext::default();

    req_context
      .req_headers
      .typed_insert(Authorization::bearer(token).unwrap());

    req_context
  }

  #[tokio::test]
  async fn validate_token_iss() {
    let jwt_options = blueprint::JwtProvider::test_value();
    let jwt_provider = JwtProvider::new(jwt_options, Arc::new(MockHttpClient));

    let valid = jwt_provider.validate(&create_jwt_auth_request(JWT_VALID_TOKEN)).await;

    assert!(valid.is_ok());

    let jwt_options = blueprint::JwtProvider { issuer: Some("me".to_owned()), ..blueprint::JwtProvider::test_value() };
    let jwt_provider = JwtProvider::new(jwt_options, Arc::new(MockHttpClient));

    let valid = jwt_provider.validate(&create_jwt_auth_request(JWT_VALID_TOKEN)).await;

    assert!(valid.is_ok());

    let jwt_options =
      blueprint::JwtProvider { issuer: Some("another".to_owned()), ..blueprint::JwtProvider::test_value() };
    let jwt_provider = JwtProvider::new(jwt_options, Arc::new(MockHttpClient));

    let error = jwt_provider
      .validate(&create_jwt_auth_request(JWT_VALID_TOKEN))
      .await
      .err();

    assert_eq!(error, Some(AuthError::Invalid));
  }

  #[tokio::test]
  async fn validate_token_aud() {
    let jwt_options = blueprint::JwtProvider::test_value();
    let jwt_provider = JwtProvider::new(jwt_options, Arc::new(MockHttpClient));

    let valid = jwt_provider.validate(&create_jwt_auth_request(JWT_VALID_TOKEN)).await;

    assert!(valid.is_ok());

    let jwt_options = blueprint::JwtProvider {
      audiences: HashSet::from_iter(["them".to_string()]),
      ..blueprint::JwtProvider::test_value()
    };
    let jwt_provider = JwtProvider::new(jwt_options, Arc::new(MockHttpClient));

    let valid = jwt_provider.validate(&create_jwt_auth_request(JWT_VALID_TOKEN)).await;

    assert!(valid.is_ok());

    let jwt_options = blueprint::JwtProvider {
      audiences: HashSet::from_iter(["anothem".to_string()]),
      ..blueprint::JwtProvider::test_value()
    };
    let jwt_provider = JwtProvider::new(jwt_options, Arc::new(MockHttpClient));

    let error = jwt_provider
      .validate(&create_jwt_auth_request(JWT_VALID_TOKEN))
      .await
      .err();

    assert_eq!(error, Some(AuthError::Invalid));
  }
}
