use std::path::PathBuf;
use std::time::Duration;

use headers::authorization::Bearer;
use headers::Authorization;
use headers::HeaderMapExt;
use jwtk::jwk::Jwk;
use jwtk::jwk::JwkSet;
use jwtk::jwk::JwkSetVerifier;
use jwtk::jwk::RemoteJwksVerifier;
use jwtk::HeaderAndClaims;
use serde::Deserialize;
use serde::Serialize;

use crate::http::RequestContext;
use crate::valid::Valid;

use super::base::{AuthError, AuthProvider};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum JwtProviderJwksOptions {
  File(PathBuf),
  Remote { url: String },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JwtProviderOptions {
  issuer: Option<String>,
  #[serde(default)]
  audiences: Vec<String>,
  jwks: JwtProviderJwksOptions,
}

enum JwksVerifier {
  Local(JwkSetVerifier),
  Remote(RemoteJwksVerifier),
}

impl From<&JwtProviderJwksOptions> for JwksVerifier {
  fn from(value: &JwtProviderJwksOptions) -> Self {
    match value {
      JwtProviderJwksOptions::File(path) => {
        // TODO: mock value
        let mut jwk = Jwk::default();
        jwk.kty = "RSA".to_owned();
        jwk.use_ = Some("sig".to_owned());
        jwk.alg = Some("RS256".to_owned());
        jwk.kid = Some("I48qMJp566SSKQogYXYtHBo9q6ZcEKHixNPeNoxV1c8".to_owned());
        jwk.n = Some("ksMb5oMlhJ_HzAebCuBG6-v5Qc4J111ur7Aux6-8SbxzqFONsf2Bw6ATG8pAfNeZ-USA3_T1mGkYTDvfoggXnxsduWV_lePZKKOq_Qp_EDdzic1bVTJQDad3CXldR3wV6UFDtMx6cCLXxPZM5n76e7ybPt0iNgwoGpJE28emMZJXrnEUFzxwFMq61UlzWEumYqW3uOUVp7r5XAF5jQ_1nQAnpHBnRFzdNPVb3E6odMGu3jgp8mkPbPMP16Fund4LVplLz8yrsE9TdVrSdYJThylRWn_BwvJ0DjUcp8ibJya86iClUlixAmBwR9NdStHwQqHwmMXMKkTXo-ytRmSUobzxX9T8ESkij6iBhQpmDMD3FbkK30Y7pUVEBBOyDfNcWOhholjOj9CRrxu9to5rc2wvufe24VlbKb9wngS_uGfK4AYvVyrcjdYMFkdqw-Mft14HwzdO2BTS0TeMDZuLmYhj_bu5_g2Zu6PH5OpIXF6Fi8_679pCG8wWAcFQrFrM0eA70wD_SqD_BXn6pWRpFXlcRy_7PWTZ3QmC7ycQFR6Wc6Px44y1xDUoq3rH0RlZkeicfvP6FRlpjFU7xF6LjAfd9ciYBZfJll6PE7zf-i_ZXEslv-tJ5-30-I4Slwj0tDrZ2Z54OgAg07AIwAiI5o4y-0vmuhUscNpfZsGAGhE".to_owned());
        jwk.e = Some("AQAB".to_owned());

        let jwks = JwkSet { keys: vec![jwk] };

        Self::Local(jwks.verifier())
      }
      JwtProviderJwksOptions::Remote { url } => todo!(),
    }
  }
}

pub struct JwtProvider {
  options: JwtProviderOptions,
  verifier: JwksVerifier,
}

impl From<JwtProviderOptions> for JwtProvider {
  fn from(options: JwtProviderOptions) -> Self {
    let verifier = JwksVerifier::from(&options.jwks);

    Self { options, verifier }
  }
}

#[async_trait::async_trait]
impl AuthProvider for JwtProvider {
  async fn validate(&self, request: &RequestContext) -> Valid<(), AuthError> {
    dbg!(&request.req_headers);
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
    // TODO: set client?
    let j = RemoteJwksVerifier::new("http://127.0.0.1:3000/jwks".into(), None, Duration::from_secs(5 * 3600));
    let c = j.verify::<()>(&token).await.unwrap();

    dbg!(&c);

    self.validate_claims(&c)
  }

  fn validate_claims(&self, parsed: &HeaderAndClaims<()>) -> Valid<(), AuthError> {
    let iss_valid = self
      .options
      .issuer
      .as_ref()
      .map(|issuer| parsed.claims().iss.as_ref().map(|iss| iss == issuer).unwrap_or(false))
      .unwrap_or(true);

    if !iss_valid {
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

  #[test]
  fn jwt_options_parse() -> Result<()> {
    let options: JwtProviderOptions = serde_json::from_value(json!({
      "jwks": {
        "file": "test-jwks.json"
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
  async fn validate_token_issuer() {
    // token with issuer = "me"
    // token is valid for 10 years. It it expired, update it =)
    let token = "eyJhbGciOiJSUzI1NiIsImtpZCI6Ikk0OHFNSnA1NjZTU0tRb2dZWFl0SEJvOXE2WmNFS0hpeE5QZU5veFYxYzgifQ.eyJleHAiOjIwMTkwNTY0NDEuMCwiaXNzIjoibWUiLCJzdWIiOiJ5b3UiLCJhdWQiOlsidGhlbSJdfQ.cU-hJgVGWxK3-IBggYBChhf3FzibBKjuDLtq2urJ99FVXIGZls0VMXjyNW7yHhLLuif_9t2N5UIUIq-hwXVv7rrGRPCGrlqKU0jsUH251Spy7_ppG5_B2LsG3cBJcwkD4AVz8qjT3AaE_vYZ4WnH-CQ-F5Vm7wiYZgbdyU8xgKoH85KAxaCdJJlYOi8mApE9_zcdmTNJrTNd9sp7PX3lXSUu9AWlrZkyO-HhVbXFunVtfduDuTeVXxP8iw1wt6171CFbPmQJU_b3xCornzyFKmhSc36yvlDfoPPclWmWeyOfFEp9lVhQm0WhfDK7GiuRtaOxD-tOvpTjpcoZBeJb7bSg2OsneyeM_33a0WoPmjHw8WIxbroJz_PrfE72_TzbcTSDttKAv_e75PE48Vvx0661miFv4Gq8RBzMl2G3pQMEVCOm83v7BpodfN_YVJcqZJjVHMA70TZQ4K3L4_i9sIK9jJFfwEDVM7nsDnUu96n4vKs1fVvAuieCIPAJrfNOUMy7TwLvhnhUARsKnzmtNNrJuDhhBx-X93AHcG3micXgnqkFdKn6-ZUZ63I2KEdmjwKmLTRrv4n4eZKrRN-OrHPI4gLxJUhmyPAHzZrikMVBcDYfALqyki5SeKkwd4v0JAm87QzR4YwMdKErr0Xa5JrZqHGe2TZgVO4hIc-KrPw";

    let jwt_options: JwtProviderOptions = serde_json::from_value(json!({
      "jwks": {
        "file": "test-jwks.json"
      }
    }))
    .unwrap();
    let jwt_provider = JwtProvider::from(jwt_options);

    let valid = jwt_provider.validate_token(token).await;

    assert!(valid.is_succeed());

    let jwt_options: JwtProviderOptions = serde_json::from_value(json!({
      "issuer": "me",
      "jwks": {
        "file": "test-jwks.json"
      }
    }))
    .unwrap();
    let jwt_provider = JwtProvider::from(jwt_options);

    let valid = jwt_provider.validate_token(token).await;

    assert!(valid.is_succeed());

    let jwt_options: JwtProviderOptions = serde_json::from_value(json!({
      "issuer": "another",
      "jwks": {
        "file": "test-jwks.json"
      }
    }))
    .unwrap();
    let jwt_provider = JwtProvider::from(jwt_options);

    let valid = jwt_provider.validate_token(token).await;

    // TODO: validate the error
    assert!(!valid.is_succeed());
  }
}
