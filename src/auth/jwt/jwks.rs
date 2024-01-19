use std::str::FromStr;

use anyhow::Result;
use derive_setters::Setters;
use jsonwebtoken::jwk::{Jwk, JwkSet};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};

use super::jwt_verify::JwtClaim;
use crate::auth::error::Error;

#[derive(Setters)]
pub struct Jwks {
  set: JwkSet,
  optional_kid: bool,
}

impl From<JwkSet> for Jwks {
  fn from(set: JwkSet) -> Self {
    Self { set, optional_kid: false }
  }
}

impl Jwks {
  fn decode_with_jwk(&self, token: &str, jwk: &Jwk) -> Result<JwtClaim, Error> {
    let key = DecodingKey::from_jwk(jwk).map_err(|_| Error::ValidationCheckFailed)?;
    let algorithm = jwk
      .common
      .key_algorithm
      .and_then(|alg| Algorithm::from_str(alg.to_string().as_str()).ok())
      .ok_or(Error::ValidationCheckFailed)?;
    let mut validation = Validation::new(algorithm);

    // will validate on our side later
    validation.validate_aud = false;

    let decoded = decode::<JwtClaim>(token, &key, &validation).map_err(|_| Error::Invalid)?;

    Ok(decoded.claims)
  }

  pub fn decode(&self, token: &str) -> Result<JwtClaim, Error> {
    let header = decode_header(token).map_err(|_| Error::Invalid)?;

    if let Some(kid) = &header.kid {
      let jwk = self.set.find(kid).ok_or(Error::ValidationCheckFailed)?;

      self.decode_with_jwk(token, jwk)
    } else {
      if !self.optional_kid {
        return Err(Error::Invalid);
      }

      // iterate over all available jwk and try to decode incoming token with it
      // if any succeeds return the data
      for jwk in self.set.keys.iter() {
        if let Ok(claims) = self.decode_with_jwk(token, jwk) {
          return Ok(claims);
        }
      }

      Err(Error::Invalid)
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::auth::jwt::jwt_verify::tests::{JWK_SET, JWT_VALID_TOKEN_NO_KID, JWT_VALID_TOKEN_WITH_KID};
  use crate::auth::jwt::jwt_verify::OneOrMany;

  #[test]
  fn test_decode_required_kid() {
    let jwks = Jwks::from(JWK_SET.clone());

    assert!(matches!(jwks.decode(""), Err(Error::Invalid)));

    let data = jwks.decode(JWT_VALID_TOKEN_WITH_KID).unwrap();

    assert!(matches!(data.aud, Some(OneOrMany::Vec(v)) if v == ["them"]));
    assert!(matches!(data.iss, Some(v) if v == "me"));

    assert!(matches!(jwks.decode(JWT_VALID_TOKEN_NO_KID), Err(Error::Invalid)));
  }

  #[test]
  fn test_decode_optional_kid() {
    let jwks = Jwks::from(JWK_SET.clone()).optional_kid(true);

    assert!(matches!(jwks.decode(""), Err(Error::Invalid)));

    let data = jwks.decode(JWT_VALID_TOKEN_WITH_KID).unwrap();

    assert!(matches!(data.aud, Some(OneOrMany::Vec(v)) if v == ["them"]));
    assert!(matches!(data.iss, Some(v) if v == "me"));

    let data = jwks.decode(JWT_VALID_TOKEN_NO_KID).unwrap();

    assert!(matches!(data.aud, Some(OneOrMany::One(v)) if v == "some"));
    assert!(matches!(data.iss, Some(v) if v == "me"));
  }
}
