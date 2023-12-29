use jwtk::Claims;

use super::JwtProviderOptions;

pub fn validate_iss(options: &JwtProviderOptions, claims: &Claims<()>) -> bool {
  options
    .issuer
    .as_ref()
    .map(|issuer| claims.iss.as_ref().map(|iss| iss == issuer).unwrap_or(false))
    .unwrap_or(true)
}

pub fn validate_aud(options: &JwtProviderOptions, claims: &Claims<()>) -> bool {
  let audiences = &options.audiences;

  if audiences.is_empty() {
    true
  } else {
    match &claims.aud {
      jwtk::OneOrMany::One(aud) => audiences.contains(aud),
      // if user token has list of aud, validate that at least one of them is inside validation set
      jwtk::OneOrMany::Vec(auds) => auds.iter().any(|aud| audiences.contains(aud)),
    }
  }
}

#[cfg(test)]
mod tests {
  use jwtk::Claims;

  use super::*;
  use crate::auth::jwt::JwtProviderOptions;

  mod iss {
    use super::*;

    #[test]
    fn validate_iss_not_defined() {
      let options = JwtProviderOptions::default();
      let mut claims = Claims::<()>::default();

      assert!(validate_iss(&options, &claims));

      claims.iss = Some("iss".to_owned());

      assert!(validate_iss(&options, &claims));
    }

    #[test]
    fn validate_iss_defined() {
      let options = JwtProviderOptions { issuer: Some("iss".to_owned()), ..Default::default() };
      let mut claims = Claims::<()>::default();

      assert!(!validate_iss(&options, &claims));

      claims.iss = Some("wrong".to_owned());

      assert!(!validate_iss(&options, &claims));

      claims.iss = Some("iss".to_owned());

      assert!(validate_iss(&options, &claims));
    }
  }

  mod aud {
    use std::collections::HashSet;

    use jwtk::OneOrMany;

    use super::*;

    #[test]
    fn validate_aud_not_defined() {
      let options = JwtProviderOptions::default();
      let mut claims = Claims::<()>::default();

      assert!(validate_aud(&options, &claims));

      claims.aud = OneOrMany::One("aud".to_owned());

      assert!(validate_aud(&options, &claims));

      claims.aud = OneOrMany::Vec(vec!["aud1".to_owned(), "aud2".to_owned()]);

      assert!(validate_aud(&options, &claims));
    }

    #[test]
    fn validate_aud_defined() {
      let options = JwtProviderOptions {
        audiences: HashSet::from_iter(["aud1".to_owned(), "aud2".to_owned()]),
        ..Default::default()
      };
      let mut claims = Claims::<()>::default();

      assert!(!validate_aud(&options, &claims));

      claims.aud = OneOrMany::One("wrong".to_owned());

      assert!(!validate_aud(&options, &claims));

      claims.aud = OneOrMany::One("aud1".to_owned());

      assert!(validate_aud(&options, &claims));

      claims.aud = OneOrMany::Vec(vec!["aud1".to_owned(), "aud5".to_owned()]);

      assert!(validate_aud(&options, &claims));
    }
  }
}
