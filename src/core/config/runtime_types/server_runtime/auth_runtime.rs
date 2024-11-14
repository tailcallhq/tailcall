use std::collections::HashSet;
use std::fmt::Debug;

use jsonwebtoken::jwk::JwkSet;
use tailcall_valid::Valid;

use crate::core::config::ConfigModule;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AuthRuntime {
    Provider(Provider),
    And(Box<AuthRuntime>, Box<AuthRuntime>),
    Or(Box<AuthRuntime>, Box<AuthRuntime>),
}

impl AuthRuntime {
    pub fn make(config_module: &ConfigModule) -> Valid<Option<AuthRuntime>, String> {
        let htpasswd = config_module.extensions().htpasswd.iter().map(|htpasswd| {
            AuthRuntime::Provider(Provider::Basic(Basic {
                htpasswd: htpasswd.content.clone(),
            }))
        });

        let jwks = config_module.extensions().jwks.iter().map(|jwks| {
            AuthRuntime::Provider(Provider::Jwt(Jwt {
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
        AuthRuntime::And(Box::new(self), Box::new(other))
    }

    pub fn or(self, other: Self) -> Self {
        AuthRuntime::Or(Box::new(self), Box::new(other))
    }
}

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

#[cfg(test)]
mod tests {
    use super::{AuthRuntime, Basic, Jwt, Provider};

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
            AuthRuntime::Provider(basic_provider_1.clone()).and(AuthRuntime::Provider(basic_provider_2.clone())),
            AuthRuntime::And(
                AuthRuntime::Provider(basic_provider_1).into(),
                AuthRuntime::Provider(basic_provider_2).into()
            )
        );
    }

    #[test]
    fn and_basic_with_jwt() {
        let basic_provider = test_basic_provider_1();
        let jwt_provider = test_jwt_provider();

        assert_eq!(
            AuthRuntime::Provider(basic_provider.clone()).and(AuthRuntime::Provider(jwt_provider.clone())),
            AuthRuntime::And(
                AuthRuntime::Provider(basic_provider).into(),
                AuthRuntime::Provider(jwt_provider).into()
            )
        );
    }

    #[test]
    fn and_nested_and_with_jwt() {
        let basic_provider_1 = test_basic_provider_1();
        let basic_provider_2 = test_basic_provider_2();
        let jwt_provider = test_jwt_provider();

        assert_eq!(
            AuthRuntime::And(
                AuthRuntime::Provider(basic_provider_1.clone()).into(),
                AuthRuntime::Provider(basic_provider_2.clone()).into()
            )
            .and(AuthRuntime::Provider(jwt_provider.clone())),
            AuthRuntime::And(
                AuthRuntime::And(
                    AuthRuntime::Provider(basic_provider_1).into(),
                    AuthRuntime::Provider(basic_provider_2).into()
                )
                .into(),
                AuthRuntime::Provider(jwt_provider).into()
            )
        );
    }
}
