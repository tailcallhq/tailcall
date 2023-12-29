mod validation;

use std::collections::HashSet;
use std::fs;
use std::num::NonZeroU64;
use std::path::PathBuf;
use std::time::Duration;

use headers::authorization::Bearer;
use headers::{Authorization, HeaderMapExt};
use jwtk::jwk::{JwkSet, JwkSetVerifier, RemoteJwksVerifier};
use jwtk::HeaderAndClaims;
use serde::{Deserialize, Serialize};
use url::Url;

use self::validation::{validate_aud, validate_iss};
use super::base::{AuthError, AuthProvider};
use crate::helpers::config_path::config_path;
use crate::http::RequestContext;
use crate::valid::{Valid, ValidationError};

mod remote {
  use std::num::NonZeroU64;

  pub fn default_max_age() -> NonZeroU64 {
    NonZeroU64::new(5 * 60 * 1000).unwrap()
  }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum JwtProviderJwksOptions {
  File(PathBuf),
  #[serde(rename_all = "camelCase")]
  Remote {
    url: String,
    #[serde(default = "remote::default_max_age")]
    max_age: NonZeroU64,
  },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct JwtProviderOptions {
  pub issuer: Option<String>,
  #[serde(default)]
  pub audiences: HashSet<String>,
  pub jwks: JwtProviderJwksOptions,
}

// only used in tests and uses mocked implementation
#[cfg(test)]
impl Default for JwtProviderOptions {
  fn default() -> Self {
    let root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut test_file = root_dir.join(file!());

    test_file.pop();
    test_file.push("tests");
    test_file.push("jwks.json");

    let jwks = JwtProviderJwksOptions::File(test_file);

    Self { issuer: Default::default(), audiences: Default::default(), jwks }
  }
}

enum JwksVerifier {
  Local(JwkSetVerifier),
  Remote(RemoteJwksVerifier),
}

impl JwksVerifier {
  pub fn parse(value: &JwtProviderJwksOptions) -> Valid<Self, String> {
    match value {
      JwtProviderJwksOptions::File(path) => Valid::from(
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
      .map(|jwks: JwkSet| Self::Local(jwks.verifier())),
      JwtProviderJwksOptions::Remote { url, max_age } => {
        Valid::from(Url::parse(url).map_err(|e| ValidationError::new(e.to_string()))).map_to(Self::Remote(
          RemoteJwksVerifier::new(
            url.to_owned(),
            // TODO: set client?
            None,
            Duration::from_millis(max_age.get()),
          ),
        ))
      }
    }
    .trace("jwks")
  }
}

impl JwksVerifier {
  async fn verify(&self, token: &str) -> jwtk::Result<HeaderAndClaims<()>> {
    match self {
      JwksVerifier::Local(verifier) => verifier.verify(token),
      JwksVerifier::Remote(verifier) => verifier.verify(token).await,
    }
  }
}

pub struct JwtProvider {
  options: JwtProviderOptions,
  verifier: JwksVerifier,
}

impl JwtProvider {
  pub fn parse(options: JwtProviderOptions) -> Valid<Self, String> {
    JwksVerifier::parse(&options.jwks).map(|verifier| Self { options, verifier })
  }
}

#[async_trait::async_trait]
impl AuthProvider for JwtProvider {
  async fn validate(&self, request: &RequestContext) -> Valid<(), AuthError> {
    let token = self.resolve_token(request);

    let Some(token) = token else {
      return Valid::fail(AuthError::Missing);
    };

    self.validate_token(&token).await
  }
}

impl JwtProvider {
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

#[cfg(test)]
mod tests {
  use anyhow::Result;
  use serde_json::json;

  use super::*;
  use crate::valid::ValidationError;

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

    assert!(matches!(options.jwks, JwtProviderJwksOptions::File(_)));

    let options: JwtProviderOptions = serde_json::from_value(json!({
      "jwks": {
        "remote": {
          "url": "http://localhost:3000"
        }
      }
    }))?;

    assert!(matches!(options.jwks, JwtProviderJwksOptions::Remote { .. }));

    Ok(())
  }

  #[tokio::test]
  async fn validate_token_iss() -> Result<()> {
    let jwt_options = JwtProviderOptions::default();
    let jwt_provider = JwtProvider::parse(jwt_options).to_result()?;

    let valid = jwt_provider.validate_token(TEST_TOKEN).await;

    assert!(valid.is_succeed());

    let jwt_options = JwtProviderOptions { issuer: Some("me".to_owned()), ..Default::default() };
    let jwt_provider = JwtProvider::parse(jwt_options).to_result()?;

    let valid = jwt_provider.validate_token(TEST_TOKEN).await;

    assert!(valid.is_succeed());

    let jwt_options = JwtProviderOptions { issuer: Some("another".to_owned()), ..Default::default() };
    let jwt_provider = JwtProvider::parse(jwt_options).to_result()?;

    let error = jwt_provider.validate_token(TEST_TOKEN).await.to_result().err();

    assert_eq!(error, Some(ValidationError::new(AuthError::ValidationFailed)));

    Ok(())
  }

  #[tokio::test]
  async fn validate_token_aud() -> Result<()> {
    let jwt_options = JwtProviderOptions::default();
    let jwt_provider = JwtProvider::parse(jwt_options).to_result()?;

    let valid = jwt_provider.validate_token(TEST_TOKEN).await;

    assert!(valid.is_succeed());

    let jwt_options = JwtProviderOptions { audiences: HashSet::from_iter(["them".to_string()]), ..Default::default() };
    let jwt_provider = JwtProvider::parse(jwt_options).to_result()?;

    let valid = jwt_provider.validate_token(TEST_TOKEN).await;

    assert!(valid.is_succeed());

    let jwt_options =
      JwtProviderOptions { audiences: HashSet::from_iter(["anothem".to_string()]), ..Default::default() };
    let jwt_provider = JwtProvider::parse(jwt_options).to_result()?;

    let error = jwt_provider.validate_token(TEST_TOKEN).await.to_result().err();

    assert_eq!(error, Some(ValidationError::new(AuthError::ValidationFailed)));
    Ok(())
  }
}
