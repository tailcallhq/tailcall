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

#[derive(Clone, Debug)]
pub enum AuthProvider {
    Basic(BasicProvider),
    Jwt(JwtProvider),
}

#[derive(Clone, Default, Debug)]
pub struct Auth(pub Vec<AuthProvider>);

impl Auth {
    pub fn make(config_module: &ConfigModule) -> Valid<Auth, String> {
        let mut providers = Vec::new();

        for htpasswd in config_module.extensions.htpasswd.iter() {
            providers.push(AuthProvider::Basic(BasicProvider {
                htpasswd: htpasswd.content.clone(),
            }))
        }

        for jwks in config_module.extensions.jwks.iter() {
            providers.push(AuthProvider::Jwt(JwtProvider {
                jwks: jwks.content.clone(),
                // TODO: read those options from link instead of using defaults
                issuer: Default::default(),
                audiences: Default::default(),
                optional_kid: Default::default(),
            }))
        }

        Valid::succeed(Auth(providers))
    }
}
