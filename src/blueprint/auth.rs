use std::collections::HashSet;
use std::fmt::Debug;

use jsonwebtoken::jwk::JwkSet;

use crate::config::ConfigModule;
use crate::valid::Valid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BasicProvider {
    pub htpasswd: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JwtProvider {
    pub issuer: Option<String>,
    pub audiences: HashSet<String>,
    pub optional_kid: bool,
    pub jwks: JwkSet,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AuthProvider {
    Basic(BasicProvider),
    Jwt(JwtProvider),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Auth {
    Single(AuthProvider),
    And(Box<Auth>, Box<Auth>),
    Or(Box<Auth>, Box<Auth>),
}

impl Auth {
    pub fn make(config_module: &ConfigModule) -> Valid<Option<Auth>, String> {
        let htpasswd = config_module.extensions.htpasswd.iter().map(|htpasswd| {
            Auth::Single(AuthProvider::Basic(BasicProvider {
                htpasswd: htpasswd.content.clone(),
            }))
        });

        let jwks = config_module.extensions.jwks.iter().map(|jwks| {
            Auth::Single(AuthProvider::Jwt(JwtProvider {
                jwks: jwks.content.clone(),
                // TODO: read those options from link instead of using defaults
                issuer: Default::default(),
                audiences: Default::default(),
                optional_kid: Default::default(),
            }))
        });

        let auth = htpasswd.chain(jwks).reduce(|left, right| left.or(right));

        Valid::succeed(auth)
    }

    pub fn and(self, other: Self) -> Self {
        Auth::And(Box::new(self), Box::new(other))
    }

    pub fn or(self, other: Self) -> Self {
        Auth::Or(Box::new(self), Box::new(other))
    }
}

#[cfg(test)]
mod tests {
    use super::{Auth, AuthProvider, BasicProvider, JwtProvider};

    #[test]
    fn test_and() {
        let basic_provider_1 = AuthProvider::Basic(BasicProvider::test_value());
        let basic_provider_2 = AuthProvider::Basic(BasicProvider::test_value());
        let jwt_provider = AuthProvider::Jwt(JwtProvider::test_value());

        assert_eq!(
            Auth::Single(basic_provider_1.clone()).and(Auth::Single(basic_provider_2.clone())),
            Auth::And(
                Auth::Single(basic_provider_1.clone()).into(),
                Auth::Single(basic_provider_2.clone()).into()
            )
        );

        assert_eq!(
            Auth::And(
                Auth::Single(basic_provider_1.clone()).into(),
                Auth::Single(basic_provider_2.clone()).into()
            )
            .and(Auth::Single(jwt_provider.clone())),
            Auth::And(
                Auth::And(
                    Auth::Single(basic_provider_1.clone()).into(),
                    Auth::Single(basic_provider_2.clone()).into()
                )
                .into(),
                Auth::Single(jwt_provider.clone()).into()
            )
        );

        assert_eq!(
            Auth::Single(jwt_provider.clone()).and(Auth::And(
                Auth::Single(basic_provider_1.clone()).into(),
                Auth::Single(basic_provider_2.clone()).into()
            )),
            Auth::And(
                Auth::Single(jwt_provider.clone()).into(),
                Auth::And(
                    Auth::Single(basic_provider_1.clone()).into(),
                    Auth::Single(basic_provider_2.clone()).into()
                )
                .into()
            )
        );

        assert_eq!(
            Auth::Or(
                Auth::Single(jwt_provider.clone()).into(),
                Auth::Single(jwt_provider.clone()).into()
            )
            .and(Auth::Or(
                Auth::Single(basic_provider_1.clone()).into(),
                Auth::Single(basic_provider_2.clone()).into()
            )),
            Auth::And(
                Auth::Or(
                    Auth::Single(jwt_provider.clone()).into(),
                    Auth::Single(jwt_provider.clone()).into()
                )
                .into(),
                Auth::Or(
                    Auth::Single(basic_provider_1.clone()).into(),
                    Auth::Single(basic_provider_2.clone()).into()
                )
                .into()
            )
        );
    }

    #[test]
    fn test_or() {
        let basic_provider_1 = AuthProvider::Basic(BasicProvider { htpasswd: "1".into() });
        let basic_provider_2 = AuthProvider::Basic(BasicProvider { htpasswd: "2".into() });
        let jwt_provider = AuthProvider::Jwt(JwtProvider::test_value());

        assert_eq!(
            Auth::Single(basic_provider_1.clone()).or(Auth::Single(basic_provider_2.clone())),
            Auth::Or(
                Auth::Single(basic_provider_1.clone()).into(),
                Auth::Single(basic_provider_2.clone()).into()
            )
        );

        assert_eq!(
            Auth::Or(
                Auth::Single(basic_provider_1.clone()).into(),
                Auth::Single(basic_provider_2.clone()).into()
            )
            .or(Auth::Single(jwt_provider.clone())),
            Auth::Or(
                Auth::Or(
                    Auth::Single(basic_provider_1.clone()).into(),
                    Auth::Single(basic_provider_2.clone()).into()
                )
                .into(),
                Auth::Single(jwt_provider.clone()).into()
            )
        );

        assert_eq!(
            Auth::Single(jwt_provider.clone()).or(Auth::Or(
                Auth::Single(basic_provider_1.clone()).into(),
                Auth::Single(basic_provider_2.clone()).into()
            )),
            Auth::Or(
                Auth::Single(jwt_provider.clone()).into(),
                Auth::Or(
                    Auth::Single(basic_provider_1.clone()).into(),
                    Auth::Single(basic_provider_2.clone()).into()
                )
                .into()
            )
        );

        assert_eq!(
            Auth::And(
                Auth::Single(jwt_provider.clone()).into(),
                Auth::Single(jwt_provider.clone()).into()
            )
            .or(Auth::And(
                Auth::Single(basic_provider_1.clone()).into(),
                Auth::Single(basic_provider_2.clone()).into()
            )),
            Auth::Or(
                Auth::And(
                    Auth::Single(jwt_provider.clone()).into(),
                    Auth::Single(jwt_provider.clone()).into()
                )
                .into(),
                Auth::And(
                    Auth::Single(basic_provider_1.clone()).into(),
                    Auth::Single(basic_provider_2.clone()).into()
                )
                .into()
            )
        );
    }
}
