use std::path::PathBuf;
use std::time::Duration;

use headers::authorization::Bearer;
use headers::Authorization;
use headers::HeaderMapExt;
use hyper::Body;
use hyper::Request;
use jwtk::jwk::Jwk;
use jwtk::jwk::JwkSet;
use jwtk::jwk::JwkSetVerifier;
use jwtk::jwk::RemoteJwksVerifier;
use serde::Deserialize;
use serde::Serialize;

use crate::valid::Valid;

use super::base::{AuthError, AuthProvider};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum JwtProviderJwksOptions {
  File(PathBuf),
  Remote { url: String },
}

#[derive(Serialize, Deserialize, Debug)]
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
  async fn validate(&mut self, request: &Request<Body>) -> Valid<(), AuthError> {
    dbg!(&request.headers());
    let token = self.resolve_token(request);

    let Some(token) = token else {
      return Valid::fail(AuthError::Missing);
    };

    // TODO: set client?
    let j = RemoteJwksVerifier::new("http://127.0.0.1:3000/jwks".into(), None, Duration::from_secs(5 * 3600));
    let c = j.verify::<()>(&token).await.unwrap();

    dbg!(&c);

    Valid::succeed(())
  }
}

impl JwtProvider {
  fn resolve_token(&self, request: &Request<Body>) -> Option<String> {
    let value = request.headers().typed_get::<Authorization<Bearer>>();

    value.map(|token| token.token().to_owned())
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
        "file": "test.txt"
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
}
