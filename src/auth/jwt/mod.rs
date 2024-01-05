mod remote_jwks;
mod validation;

use std::fs;
use std::sync::Arc;
use std::time::Duration;

use headers::authorization::Bearer;
use headers::{Authorization, HeaderMapExt};
use jwtk::jwk::{JwkSet, JwkSetVerifier};
use jwtk::HeaderAndClaims;
use url::Url;

use self::remote_jwks::RemoteJwksVerifier;
use self::validation::{validate_aud, validate_iss};
use super::base::{AuthError, AuthProviderTrait};
use crate::config::{JwksOptions, JwksVerifierOptions, JwtProviderOptions};
use crate::helpers::config_path::config_path;
use crate::http::{HttpClient, RequestContext};
use crate::valid::{Valid, ValidationError};

// only used in tests and uses mocked implementation
#[cfg(test)]
impl Default for JwtProviderOptions {
  fn default() -> Self {
    use std::path::PathBuf;

    let root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut test_file = root_dir.join(file!());

    test_file.pop();
    test_file.push("tests");
    test_file.push("jwks.json");

    let jwks = JwksOptions { optional_kid: false, verifier: JwksVerifierOptions::File(test_file) };

    Self { issuer: Default::default(), audiences: Default::default(), jwks }
  }
}

enum JwksVerifier {
  Local(JwkSetVerifier),
  Remote(RemoteJwksVerifier),
}

impl JwksVerifier {
  pub fn new(options: &JwksOptions, client: Arc<dyn HttpClient>) -> Valid<Self, String> {
    match &options.verifier {
      JwksVerifierOptions::File(path) => Valid::from(
        config_path(path)
          .and_then(fs::read_to_string)
          .map_err(|e| ValidationError::new(e.to_string())),
      )
      .and_then(|file| {
        let de = &mut serde_json::Deserializer::from_str(&file);

        Valid::from(serde_path_to_error::deserialize(de).map_err(ValidationError::from))
      })
      .trace(&format!("{}", path.display()))
      .trace("file")
      .map(|jwks: JwkSet| {
        let mut verifier = jwks.verifier();

        verifier.set_require_kid(!options.optional_kid);

        Self::Local(verifier)
      }),
      JwksVerifierOptions::Remote { url, max_age } => {
        Valid::from(Url::parse(url).map_err(|e| ValidationError::new(e.to_string()))).map(|url| {
          let mut verifier = RemoteJwksVerifier::new(url, client, Duration::from_millis(max_age.get()));

          verifier.set_require_kid(!options.optional_kid);

          Self::Remote(verifier)
        })
      }
    }
    .trace("jwks")
  }

  async fn verify(&self, token: &str) -> Result<HeaderAndClaims<()>, AuthError> {
    match self {
      JwksVerifier::Local(verifier) => verifier.verify(token).map_err(|_| AuthError::ValidationFailed),
      JwksVerifier::Remote(verifier) => verifier.verify(token).await,
    }
  }
}

pub struct JwtProvider {
  options: JwtProviderOptions,
  verifier: JwksVerifier,
}

impl JwtProvider {
  pub fn new(options: JwtProviderOptions, client: Arc<dyn HttpClient>) -> Valid<Self, String> {
    JwksVerifier::new(&options.jwks, client).map(|verifier| Self { options, verifier })
  }

  fn resolve_token(&self, request: &RequestContext) -> Option<String> {
    let value = request.req_headers.typed_get::<Authorization<Bearer>>();

    value.map(|token| token.token().to_owned())
  }

  async fn validate_token(&self, token: &str) -> Valid<(), AuthError> {
    let verification = self.verifier.verify(token).await;

    Valid::from(verification.map_err(|_| ValidationError::new(AuthError::ValidationNotAccessible)))
      .and_then(|v| self.validate_claims(&v))
  }

  fn validate_claims(&self, parsed: &HeaderAndClaims<()>) -> Valid<(), AuthError> {
    let claims = parsed.claims();

    if !validate_iss(&self.options, claims) || !validate_aud(&self.options, claims) {
      return Valid::fail(AuthError::ValidationFailed);
    }

    Valid::succeed(())
  }
}

impl AuthProviderTrait for JwtProvider {
  async fn validate(&self, request: &RequestContext) -> Valid<(), AuthError> {
    let token = self.resolve_token(request);

    let Some(token) = token else {
      return Valid::fail(AuthError::Missing);
    };

    self.validate_token(&token).await
  }
}

#[cfg(test)]
mod tests {
  use std::collections::HashSet;

  use anyhow::Result;
  use serde_json::json;

  use super::*;
  use crate::valid::ValidationError;

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
  const TEST_TOKEN: &str = "eyJhbGciOiJSUzI1NiIsImtpZCI6Ikk0OHFNSnA1NjZTU0tRb2dZWFl0SEJvOXE2WmNFS0hpeE5QZU5veFYxYzgifQ.eyJleHAiOjIwMTkwNTY0NDEuMCwiaXNzIjoibWUiLCJzdWIiOiJ5b3UiLCJhdWQiOlsidGhlbSJdfQ.cU-hJgVGWxK3-IBggYBChhf3FzibBKjuDLtq2urJ99FVXIGZls0VMXjyNW7yHhLLuif_9t2N5UIUIq-hwXVv7rrGRPCGrlqKU0jsUH251Spy7_ppG5_B2LsG3cBJcwkD4AVz8qjT3AaE_vYZ4WnH-CQ-F5Vm7wiYZgbdyU8xgKoH85KAxaCdJJlYOi8mApE9_zcdmTNJrTNd9sp7PX3lXSUu9AWlrZkyO-HhVbXFunVtfduDuTeVXxP8iw1wt6171CFbPmQJU_b3xCornzyFKmhSc36yvlDfoPPclWmWeyOfFEp9lVhQm0WhfDK7GiuRtaOxD-tOvpTjpcoZBeJb7bSg2OsneyeM_33a0WoPmjHw8WIxbroJz_PrfE72_TzbcTSDttKAv_e75PE48Vvx0661miFv4Gq8RBzMl2G3pQMEVCOm83v7BpodfN_YVJcqZJjVHMA70TZQ4K3L4_i9sIK9jJFfwEDVM7nsDnUu96n4vKs1fVvAuieCIPAJrfNOUMy7TwLvhnhUARsKnzmtNNrJuDhhBx-X93AHcG3micXgnqkFdKn6-ZUZ63I2KEdmjwKmLTRrv4n4eZKrRN-OrHPI4gLxJUhmyPAHzZrikMVBcDYfALqyki5SeKkwd4v0JAm87QzR4YwMdKErr0Xa5JrZqHGe2TZgVO4hIc-KrPw";

  #[test]
  fn jwt_options_parse() -> Result<()> {
    let options: JwtProviderOptions = serde_json::from_value(json!({
      "jwks": {
        "file": "tests/server/config/jwks.json"
      }
    }))?;

    assert!(matches!(
      options.jwks,
      JwksOptions { optional_kid: false, verifier: JwksVerifierOptions::File(_) }
    ));

    let options: JwtProviderOptions = serde_json::from_value(json!({
      "jwks": {
        "optionalKid": true,
        "remote": {
          "url": "http://localhost:3000"
        }
      }
    }))?;

    assert!(matches!(
      options.jwks,
      JwksOptions { optional_kid: true, verifier: JwksVerifierOptions::Remote { .. } }
    ));

    Ok(())
  }

  #[tokio::test]
  async fn validate_token_iss() -> Result<()> {
    let jwt_options = JwtProviderOptions::default();
    let jwt_provider = JwtProvider::new(jwt_options, Arc::new(MockHttpClient)).to_result()?;

    let valid = jwt_provider.validate_token(TEST_TOKEN).await;

    assert!(valid.is_succeed());

    let jwt_options = JwtProviderOptions { issuer: Some("me".to_owned()), ..Default::default() };
    let jwt_provider = JwtProvider::new(jwt_options, Arc::new(MockHttpClient)).to_result()?;

    let valid = jwt_provider.validate_token(TEST_TOKEN).await;

    assert!(valid.is_succeed());

    let jwt_options = JwtProviderOptions { issuer: Some("another".to_owned()), ..Default::default() };
    let jwt_provider = JwtProvider::new(jwt_options, Arc::new(MockHttpClient)).to_result()?;

    let error = jwt_provider.validate_token(TEST_TOKEN).await.to_result().err();

    assert_eq!(error, Some(ValidationError::new(AuthError::ValidationFailed)));

    Ok(())
  }

  #[tokio::test]
  async fn validate_token_aud() -> Result<()> {
    let jwt_options = JwtProviderOptions::default();
    let jwt_provider = JwtProvider::new(jwt_options, Arc::new(MockHttpClient)).to_result()?;

    let valid = jwt_provider.validate_token(TEST_TOKEN).await;

    assert!(valid.is_succeed());

    let jwt_options = JwtProviderOptions { audiences: HashSet::from_iter(["them".to_string()]), ..Default::default() };
    let jwt_provider = JwtProvider::new(jwt_options, Arc::new(MockHttpClient)).to_result()?;

    let valid = jwt_provider.validate_token(TEST_TOKEN).await;

    assert!(valid.is_succeed());

    let jwt_options =
      JwtProviderOptions { audiences: HashSet::from_iter(["anothem".to_string()]), ..Default::default() };
    let jwt_provider = JwtProvider::new(jwt_options, Arc::new(MockHttpClient)).to_result()?;

    let error = jwt_provider.validate_token(TEST_TOKEN).await.to_result().err();

    assert_eq!(error, Some(ValidationError::new(AuthError::ValidationFailed)));
    Ok(())
  }
}
