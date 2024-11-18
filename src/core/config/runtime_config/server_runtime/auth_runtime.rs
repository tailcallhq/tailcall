use std::collections::HashSet;
use std::fmt::Debug;

use jsonwebtoken::jwk::JwkSet;
use tailcall_valid::Valid;

use crate::core::config::ConfigModule;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AuthRuntime {
    Provider(AuthProviderRuntime),
    And(Box<AuthRuntime>, Box<AuthRuntime>),
    Or(Box<AuthRuntime>, Box<AuthRuntime>),
}

impl AuthRuntime {
    pub fn make(config_module: &ConfigModule) -> Valid<Option<AuthRuntime>, String> {
        let htpasswd = config_module.extensions().htpasswd.iter().map(|htpasswd| {
            AuthRuntime::Provider(AuthProviderRuntime::Basic(BasicRuntime {
                htpasswd: htpasswd.content.clone(),
            }))
        });

        let jwks = config_module.extensions().jwks.iter().map(|jwks| {
            AuthRuntime::Provider(AuthProviderRuntime::Jwt(JwtRuntime {
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
pub struct BasicRuntime {
    pub htpasswd: String,
}

// testuser1:password123
// testuser2:mypassword
// testuser3:abc123
pub static HTPASSWD_TEST: &str = "
testuser1:$apr1$e3dp9qh2$fFIfHU9bilvVZBl8TxKzL/
testuser2:$2y$10$wJ/mZDURcAOBIrswCAKFsO0Nk7BpHmWl/XuhF7lNm3gBAFH3ofsuu
testuser3:{SHA}Y2fEjdGT1W6nsLqtJbGUVeUp9e4=
";

impl BasicRuntime {
    pub fn test_value() -> Self {
        Self { htpasswd: HTPASSWD_TEST.to_owned() }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JwtRuntime {
    pub issuer: Option<String>,
    pub audiences: HashSet<String>,
    pub optional_kid: bool,
    pub jwks: JwkSet,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AuthProviderRuntime {
    Basic(BasicRuntime),
    Jwt(JwtRuntime),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_basic_provider_1() -> AuthProviderRuntime {
        AuthProviderRuntime::Basic(BasicRuntime { htpasswd: "1".into() })
    }

    fn test_basic_provider_2() -> AuthProviderRuntime {
        AuthProviderRuntime::Basic(BasicRuntime { htpasswd: "2".into() })
    }

    fn test_jwt_provider() -> AuthProviderRuntime {
        AuthProviderRuntime::Jwt(JwtRuntime::test_value())
    }

    #[test]
    fn and_basic_with_basic() {
        let basic_provider_1 = test_basic_provider_1();
        let basic_provider_2 = test_basic_provider_2();

        assert_eq!(
            AuthRuntime::Provider(basic_provider_1.clone())
                .and(AuthRuntime::Provider(basic_provider_2.clone())),
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
            AuthRuntime::Provider(basic_provider.clone())
                .and(AuthRuntime::Provider(jwt_provider.clone())),
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
