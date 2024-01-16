use super::{JwtClaims, OneOrMany};
use crate::blueprint;

pub fn validate_iss(options: &blueprint::JwtProvider, claims: &JwtClaims) -> bool {
  options
    .issuer
    .as_ref()
    .map(|issuer| claims.iss.as_ref().map(|iss| iss == issuer).unwrap_or(false))
    .unwrap_or(true)
}

pub fn validate_aud(options: &blueprint::JwtProvider, claims: &JwtClaims) -> bool {
  let audiences = &options.audiences;

  if audiences.is_empty() {
    true
  } else {
    let Some(aud) = &claims.aud else { return false };

    match aud {
      OneOrMany::One(aud) => audiences.contains(aud),
      // if user token has list of aud, validate that at least one of them is inside validation set
      OneOrMany::Vec(auds) => auds.iter().any(|aud| audiences.contains(aud)),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod iss {
    use super::*;
    use crate::blueprint::JwtProvider;

    #[test]
    fn validate_iss_not_defined() {
      let options = JwtProvider::test_value();
      let mut claims = JwtClaims::default();

      assert!(validate_iss(&options, &claims));

      claims.iss = Some("iss".to_owned());

      assert!(validate_iss(&options, &claims));
    }

    #[test]
    fn validate_iss_defined() {
      let options = JwtProvider { issuer: Some("iss".to_owned()), ..JwtProvider::test_value() };
      let mut claims = JwtClaims::default();

      assert!(!validate_iss(&options, &claims));

      claims.iss = Some("wrong".to_owned());

      assert!(!validate_iss(&options, &claims));

      claims.iss = Some("iss".to_owned());

      assert!(validate_iss(&options, &claims));
    }
  }

  mod aud {
    use std::collections::HashSet;

    use super::*;
    use crate::blueprint::JwtProvider;

    #[test]
    fn validate_aud_not_defined() {
      let options = JwtProvider::test_value();
      let mut claims = JwtClaims::default();

      assert!(validate_aud(&options, &claims));

      claims.aud = Some(OneOrMany::One("aud".to_owned()));

      assert!(validate_aud(&options, &claims));

      claims.aud = Some(OneOrMany::Vec(vec!["aud1".to_owned(), "aud2".to_owned()]));

      assert!(validate_aud(&options, &claims));
    }

    #[test]
    fn validate_aud_defined() {
      let options = JwtProvider {
        audiences: HashSet::from_iter(["aud1".to_owned(), "aud2".to_owned()]),
        ..JwtProvider::test_value()
      };
      let mut claims = JwtClaims::default();

      assert!(!validate_aud(&options, &claims));

      claims.aud = Some(OneOrMany::One("wrong".to_owned()));

      assert!(!validate_aud(&options, &claims));

      claims.aud = Some(OneOrMany::One("aud1".to_owned()));

      assert!(validate_aud(&options, &claims));

      claims.aud = Some(OneOrMany::Vec(vec!["aud1".to_owned(), "aud5".to_owned()]));

      assert!(validate_aud(&options, &claims));
    }
  }
}
