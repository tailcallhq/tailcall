use std::collections::HashSet;
use std::time::Duration;

use crate::config;
use crate::directive::DirectiveCodec;
use crate::mustache::Mustache;
use crate::valid::{Valid, ValidationError, Validator};

#[derive(Debug, Clone)]
pub struct BasicProvider {
    pub htpasswd: Mustache,
}

#[derive(Debug, Clone)]
pub enum Jwks {
    Local(Mustache),
    Remote { url: Mustache, max_age: Duration },
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
    Basic(BasicProvider),
    Jwt(JwtProvider),
}

#[derive(Clone, Debug)]
pub struct AuthEntry {
    pub provider: AuthProvider,
}

#[derive(Clone, Default, Debug)]
pub struct Auth(pub Vec<AuthEntry>);

impl Auth {
    pub fn make(auth: &config::Auth) -> Valid<Auth, String> {
        Valid::from_iter(&auth.0, |input| {
            let provider = match &input.provider {
                config::AuthProvider::Basic(basic) => to_basic(basic.clone())
                    .map(AuthProvider::Basic)
                    .trace(config::Basic::directive_name().as_str()),
                config::AuthProvider::Jwt(jwt) => to_jwt(jwt.clone())
                    .map(AuthProvider::Jwt)
                    .trace(config::Jwt::directive_name().as_str()),
            };

            provider.map(|provider| AuthEntry { provider })
        })
        .map(Auth)
        .trace(config::Auth::directive_name().as_str())
    }
}

fn to_basic(options: config::Basic) -> Valid<BasicProvider, String> {
    match options {
        config::Basic::Htpasswd(data) => {
            Valid::from(Mustache::parse(&data).map_err(|e| ValidationError::new(e.to_string())))
                .map(|htpasswd| BasicProvider { htpasswd })
        }
    }
}

fn to_jwt(options: config::Jwt) -> Valid<JwtProvider, String> {
    let jwks = &options.jwks;

    let jwks_valid = match &jwks {
        config::Jwks::Data(data) => {
            Valid::from(Mustache::parse(data).map_err(|e| ValidationError::new(e.to_string())))
                .map(Jwks::Local)
        }
        config::Jwks::Remote { url, max_age } => {
            Valid::from(Mustache::parse(url).map_err(|e| ValidationError::new(e.to_string())))
                .map(|url| Jwks::Remote { url, max_age: Duration::from_millis(max_age.get()) })
        }
    };

    jwks_valid.map(|jwks| JwtProvider {
        issuer: options.issuer,
        audiences: options.audiences,
        optional_kid: options.optional_kid,
        jwks,
    })
}
