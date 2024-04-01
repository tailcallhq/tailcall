use std::collections::HashSet;
use std::fmt::Debug;

use jsonwebtoken::jwk::JwkSet;

use crate::config::ConfigModule;
use crate::valid::Valid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Basic {
    pub htpasswd: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Jwt {
    pub issuer: Option<String>,
    pub audiences: HashSet<String>,
    pub optional_kid: bool,
    pub jwks: JwkSet,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Provider {
    Basic(Basic),
    Jwt(Jwt),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Auth {
    Provider(Provider),
    And(Box<Auth>, Box<Auth>),
    Or(Box<Auth>, Box<Auth>),
}

impl Auth {
    pub fn make(config_module: &ConfigModule) -> Valid<Option<Auth>, String> {
        let htpasswd = config_module.extensions.htpasswd.iter().map(|htpasswd| {
            Auth::Provider(Provider::Basic(Basic {
                htpasswd: htpasswd.content.clone(),
            }))
        });

        let jwks = config_module.extensions.jwks.iter().map(|jwks| {
            Auth::Provider(Provider::Jwt(Jwt {
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
    use super::{Auth, Basic, Jwt, Provider};

    #[test]
    fn test_and() {
        let basic_provider_1 = Provider::Basic(Basic::test_value());
        let basic_provider_2 = Provider::Basic(Basic::test_value());
        let jwt_provider = Provider::Jwt(Jwt::test_value());

        assert_eq!(
            Auth::Provider(basic_provider_1.clone()).and(Auth::Provider(basic_provider_2.clone())),
            Auth::And(
                Auth::Provider(basic_provider_1.clone()).into(),
                Auth::Provider(basic_provider_2.clone()).into()
            )
        );

        assert_eq!(
            Auth::And(
                Auth::Provider(basic_provider_1.clone()).into(),
                Auth::Provider(basic_provider_2.clone()).into()
            )
            .and(Auth::Provider(jwt_provider.clone())),
            Auth::And(
                Auth::And(
                    Auth::Provider(basic_provider_1.clone()).into(),
                    Auth::Provider(basic_provider_2.clone()).into()
                )
                .into(),
                Auth::Provider(jwt_provider.clone()).into()
            )
        );

        assert_eq!(
            Auth::Provider(jwt_provider.clone()).and(Auth::And(
                Auth::Provider(basic_provider_1.clone()).into(),
                Auth::Provider(basic_provider_2.clone()).into()
            )),
            Auth::And(
                Auth::Provider(jwt_provider.clone()).into(),
                Auth::And(
                    Auth::Provider(basic_provider_1.clone()).into(),
                    Auth::Provider(basic_provider_2.clone()).into()
                )
                .into()
            )
        );

        assert_eq!(
            Auth::Or(
                Auth::Provider(jwt_provider.clone()).into(),
                Auth::Provider(jwt_provider.clone()).into()
            )
            .and(Auth::Or(
                Auth::Provider(basic_provider_1.clone()).into(),
                Auth::Provider(basic_provider_2.clone()).into()
            )),
            Auth::And(
                Auth::Or(
                    Auth::Provider(jwt_provider.clone()).into(),
                    Auth::Provider(jwt_provider.clone()).into()
                )
                .into(),
                Auth::Or(
                    Auth::Provider(basic_provider_1.clone()).into(),
                    Auth::Provider(basic_provider_2.clone()).into()
                )
                .into()
            )
        );
    }

    #[test]
    fn test_or() {
        let basic_provider_1 = Provider::Basic(Basic { htpasswd: "1".into() });
        let basic_provider_2 = Provider::Basic(Basic { htpasswd: "2".into() });
        let jwt_provider = Provider::Jwt(Jwt::test_value());

        assert_eq!(
            Auth::Provider(basic_provider_1.clone()).or(Auth::Provider(basic_provider_2.clone())),
            Auth::Or(
                Auth::Provider(basic_provider_1.clone()).into(),
                Auth::Provider(basic_provider_2.clone()).into()
            )
        );

        assert_eq!(
            Auth::Or(
                Auth::Provider(basic_provider_1.clone()).into(),
                Auth::Provider(basic_provider_2.clone()).into()
            )
            .or(Auth::Provider(jwt_provider.clone())),
            Auth::Or(
                Auth::Or(
                    Auth::Provider(basic_provider_1.clone()).into(),
                    Auth::Provider(basic_provider_2.clone()).into()
                )
                .into(),
                Auth::Provider(jwt_provider.clone()).into()
            )
        );

        assert_eq!(
            Auth::Provider(jwt_provider.clone()).or(Auth::Or(
                Auth::Provider(basic_provider_1.clone()).into(),
                Auth::Provider(basic_provider_2.clone()).into()
            )),
            Auth::Or(
                Auth::Provider(jwt_provider.clone()).into(),
                Auth::Or(
                    Auth::Provider(basic_provider_1.clone()).into(),
                    Auth::Provider(basic_provider_2.clone()).into()
                )
                .into()
            )
        );

        assert_eq!(
            Auth::And(
                Auth::Provider(jwt_provider.clone()).into(),
                Auth::Provider(jwt_provider.clone()).into()
            )
            .or(Auth::And(
                Auth::Provider(basic_provider_1.clone()).into(),
                Auth::Provider(basic_provider_2.clone()).into()
            )),
            Auth::Or(
                Auth::And(
                    Auth::Provider(jwt_provider.clone()).into(),
                    Auth::Provider(jwt_provider.clone()).into()
                )
                .into(),
                Auth::And(
                    Auth::Provider(basic_provider_1.clone()).into(),
                    Auth::Provider(basic_provider_2.clone()).into()
                )
                .into()
            )
        );
    }
}
