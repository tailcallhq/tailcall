use std::collections::HashSet;
use std::fmt::Debug;

use jsonwebtoken::jwk::JwkSet;

use crate::config::ConfigModule;
use crate::valid::Valid;

#[derive(Clone, Debug)]
pub struct BasicProvider {
    pub htpasswd: String,
}

#[derive(Clone, Debug)]
pub struct JwtProvider {
    pub issuer: Option<String>,
    pub audiences: HashSet<String>,
    pub optional_kid: bool,
    pub jwks: JwkSet,
}

#[derive(Clone, Debug, Default)]
pub enum Auth {
    Basic(BasicProvider),
    Jwt(JwtProvider),
    And(Box<Auth>, Box<Auth>),
    Or(Box<Auth>, Box<Auth>),
    #[default]
    Empty,
}

impl Auth {
    pub fn make(config_module: &ConfigModule) -> Valid<Auth, String> {
        let mut auth = Auth::default();

        for htpasswd in config_module.extensions.htpasswd.iter() {
            auth = auth.or(Auth::Basic(BasicProvider {
                htpasswd: htpasswd.content.clone(),
            }))
        }

        for jwks in config_module.extensions.jwks.iter() {
            auth = auth.or(Auth::Jwt(JwtProvider {
                jwks: jwks.content.clone(),
                // TODO: read those options from link instead of using defaults
                issuer: Default::default(),
                audiences: Default::default(),
                optional_kid: Default::default(),
            }))
        }

        Valid::succeed(auth)
    }

    pub fn or(self, other: Self) -> Self {
        Auth::Or(Box::new(self), Box::new(other))
    }
}
