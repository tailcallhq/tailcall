use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::time::Duration;

use jwtk::jwk::JwkSet;
use url::Url;

use super::init_context::InitContext;
use crate::config;
use crate::helpers::config_path::config_path;
use crate::mustache::Mustache;
use crate::valid::{Valid, ValidationError};

#[derive(Debug)]
pub enum Jwks {
  Local(JwkSet),
  Remote { url: Url, max_age: Duration },
}

impl Clone for Jwks {
  fn clone(&self) -> Self {
    match self {
      Self::Local(jwks) => {
        // TODO: hack to clone JwkSet
        // maybe try another library that has built in cloning?
        Self::Local(serde_json::from_value(serde_json::to_value(jwks).unwrap()).unwrap())
      }
      Self::Remote { url, max_age } => Self::Remote { url: url.clone(), max_age: *max_age },
    }
  }
}

#[derive(Clone, Debug)]
pub struct JwtProvider {
  pub issuer: Option<String>,
  pub audiences: HashSet<String>,
  pub optional_kid: bool,
  pub jwks: Jwks,
}

#[derive(Clone, Debug)]
pub enum AuthProvider {
  JWT(JwtProvider),
}

#[derive(Clone, Debug)]
pub struct AuthEntry {
  pub id: String,
  pub provider: AuthProvider,
}

#[derive(Clone, Default, Debug)]
pub struct Auth(pub Vec<AuthEntry>);

fn to_jwt(init_context: &InitContext, options: config::JwtProvider) -> Valid<JwtProvider, String> {
  let jwks = &options.jwks;

  let jwks_valid = match &jwks {
    config::Jwks::Const(data) => Valid::from(Mustache::parse(data).map_err(|e| ValidationError::new(e.to_string())))
      .and_then(|tmpl| {
        let data = tmpl.render(init_context);
        let de = &mut serde_json::Deserializer::from_str(&data);

        Valid::from(serde_path_to_error::deserialize(de).map_err(ValidationError::from))
          .trace("const")
          .map(|jwks: JwkSet| Jwks::Local(jwks))
      }),
    config::Jwks::File(path) => Valid::from(Mustache::parse(path).map_err(|e| ValidationError::new(e.to_string())))
      .and_then(|path| {
        let path = path.render(init_context);
        let path = Path::new(&path);
        Valid::from(
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
        .map(|jwks: JwkSet| Jwks::Local(jwks))
      }),
    config::Jwks::Remote { url, max_age } => {
      Valid::from(Mustache::parse(url).map_err(|e| ValidationError::new(e.to_string()))).and_then(|url| {
        let url = url.render(init_context);

        Valid::from(Url::parse(&url).map_err(|e| ValidationError::new(e.to_string())))
          .map(|url| Jwks::Remote { url, max_age: Duration::from_millis(max_age.get()) })
      })
    }
  }
  .trace("jwks");

  jwks_valid.map(|jwks| JwtProvider {
    issuer: options.issuer,
    audiences: options.audiences,
    optional_kid: options.optional_kid,
    jwks,
  })
}

pub fn to_auth(init_context: &InitContext, auth: &config::Auth) -> Valid<Auth, String> {
  Valid::from_iter(&auth.0, |input| {
    let provider = match &input.provider {
      config::AuthProvider::JWT(jwt) => to_jwt(init_context, jwt.clone())
        .map(AuthProvider::JWT)
        .trace("JWT"),
    };

    provider.map(|provider| AuthEntry { id: input.id.clone(), provider })
  })
  .map(Auth)
  .trace("auth")
}
