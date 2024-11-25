use std::collections::HashSet;
use std::fmt::Debug;

use jsonwebtoken::jwk::JwkSet;

use crate::core::config::{ConfigModule, Content};

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

impl From<Content<String>> for Content<Provider> {
    fn from(content: Content<String>) -> Self {
        Content {
            id: content.id,
            content: Provider::Basic(Basic { htpasswd: content.content }),
        }
    }
}

impl From<Content<JwkSet>> for Content<Provider> {
    fn from(content: Content<JwkSet>) -> Self {
        Content {
            id: content.id,
            content: Provider::Jwt(Jwt {
                jwks: content.content,
                issuer: None,
                audiences: HashSet::new(),
                optional_kid: false,
            }),
        }
    }
}

impl Provider {
    /// Used to collect all auth providers from the config module
    pub fn from_config(config_module: &ConfigModule) -> Vec<Content<Provider>> {
        config_module
            .extensions()
            .htpasswd
            .iter()
            .map(|htpasswd| htpasswd.clone().into())
            .chain(
                config_module
                    .extensions()
                    .jwks
                    .iter()
                    .map(|jwks| jwks.clone().into()),
            )
            .collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Auth {
    Provider(Provider),
    And(Box<Auth>, Box<Auth>),
    Or(Box<Auth>, Box<Auth>),
}

impl Auth {
    pub fn from_config(config_module: &ConfigModule) -> Option<Auth> {
        Provider::from_config(config_module)
            .into_iter()
            .map(|c| Auth::Provider(c.content))
            .reduce(|left, right| left.and(right))
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
