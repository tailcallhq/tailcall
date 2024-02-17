use std::collections::HashSet;
use std::time::Duration;

pub use config::Basic as BasicProvider;
use jsonwebtoken::jwk::JwkSet;
use url::Url;

use crate::config;
use crate::directive::DirectiveCodec;
use crate::valid::{Valid, ValidationError, Validator};

#[derive(Debug, Clone)]
pub enum Jwks {
    Local(JwkSet),
    Remote { url: Url, max_age: Duration },
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
                config::AuthProvider::Basic(basic) => {
                    Valid::succeed(AuthProvider::Basic(basic.clone()))
                }
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

fn to_jwt(options: config::Jwt) -> Valid<JwtProvider, String> {
    let jwks_valid = match options.jwks {
        config::Jwks::Data(jwks) => {
            if jwks.is_empty() {
                Valid::<Jwks, _>::fail("JWKS data is empty".to_owned());
            }

            let de = &mut serde_json::Deserializer::from_str(&jwks);

            Valid::from(
                serde_path_to_error::deserialize(de)
                    .map_err(|err| ValidationError::new(err.to_string())),
            )
            .map(|jwks: JwkSet| Jwks::Local(jwks))
        }
        config::Jwks::Remote { url, max_age } => {
            Valid::from(Url::parse(&url).map_err(|err| ValidationError::new(err.to_string())))
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
