use std::collections::HashSet;
use std::fmt::Debug;

use jsonwebtoken::jwk::JwkSet;

use crate::core::config::ConfigModule;
use crate::core::valid::Valid;

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
        let htpasswd = config_module.extensions().htpasswd.iter().map(|htpasswd| {
            Auth::Provider(Provider::Basic(Basic {
                htpasswd: htpasswd.content.clone(),
            }))
        });

        let jwks = config_module.extensions().jwks.iter().map(|jwks| {
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

    fn test_basic_provider_1() -> Provider {
        Provider::Basic(Basic { htpasswd: "1".into() })
    }

    fn test_basic_provider_2() -> Provider {
        Provider::Basic(Basic { htpasswd: "2".into() })
    }

    fn test_jwt_provider() -> Provider {
        Provider::Jwt(Jwt::test_value())
    }

    #[test]
    fn and_basic_with_basic() {
        let basic_provider_1 = test_basic_provider_1();
        let basic_provider_2 = test_basic_provider_2();

        assert_eq!(
            Auth::Provider(basic_provider_1.clone()).and(Auth::Provider(basic_provider_2.clone())),
            Auth::And(
                Auth::Provider(basic_provider_1).into(),
                Auth::Provider(basic_provider_2).into()
            )
        );
    }

    #[test]
    fn and_basic_with_jwt() {
        let basic_provider = test_basic_provider_1();
        let jwt_provider = test_jwt_provider();

        assert_eq!(
            Auth::Provider(basic_provider.clone()).and(Auth::Provider(jwt_provider.clone())),
            Auth::And(
                Auth::Provider(basic_provider).into(),
                Auth::Provider(jwt_provider).into()
            )
        );
    }

    #[test]
    fn and_nested_and_with_jwt() {
        let basic_provider_1 = test_basic_provider_1();
        let basic_provider_2 = test_basic_provider_2();
        let jwt_provider = test_jwt_provider();

        assert_eq!(
            Auth::And(
                Auth::Provider(basic_provider_1.clone()).into(),
                Auth::Provider(basic_provider_2.clone()).into()
            )
            .and(Auth::Provider(jwt_provider.clone())),
            Auth::And(
                Auth::And(
                    Auth::Provider(basic_provider_1).into(),
                    Auth::Provider(basic_provider_2).into()
                )
                .into(),
                Auth::Provider(jwt_provider).into()
            )
        );
    }
}
