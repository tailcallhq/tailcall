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
    All(Vec<Auth>),
    Any(Vec<Auth>),
}

impl Auth {
    pub fn make(config_module: &ConfigModule) -> Valid<Option<Auth>, String> {
        if !config_module.extensions.has_auth() {
            return Valid::succeed(None);
        }

        let mut auth = Auth::Any(Vec::new());

        for htpasswd in config_module.extensions.htpasswd.iter() {
            auth = auth.or(Auth::Single(AuthProvider::Basic(BasicProvider {
                htpasswd: htpasswd.content.clone(),
            })));
        }

        for jwks in config_module.extensions.jwks.iter() {
            auth = auth.or(Auth::Single(AuthProvider::Jwt(JwtProvider {
                jwks: jwks.content.clone(),
                // TODO: read those options from link instead of using defaults
                issuer: Default::default(),
                audiences: Default::default(),
                optional_kid: Default::default(),
            })));
        }

        Valid::succeed(Some(auth))
    }

    pub fn and(self, other: Self) -> Self {
        let v = match (self, other) {
            (Auth::All(mut v1), Auth::All(mut v2)) => {
                v1.append(&mut v2);
                v1
            }
            (Auth::All(mut v), other) | (other, Auth::All(mut v)) => {
                v.push(other);
                v
            }
            (this, other) => vec![this, other],
        };

        Auth::All(v)
    }

    pub fn or(self, other: Self) -> Self {
        let v = match (self, other) {
            (Auth::Any(mut v1), Auth::Any(mut v2)) => {
                v1.append(&mut v2);
                v1
            }
            (Auth::Any(mut v), other) | (other, Auth::Any(mut v)) => {
                v.push(other);
                v
            }
            (this, other) => vec![this, other],
        };

        Auth::Any(v)
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
            Auth::All(vec![
                Auth::Single(basic_provider_1.clone()),
                Auth::Single(basic_provider_2.clone())
            ])
        );

        assert_eq!(
            Auth::All(vec![
                Auth::Single(basic_provider_1.clone()),
                Auth::Single(basic_provider_2.clone())
            ])
            .and(Auth::Single(jwt_provider.clone())),
            Auth::All(vec![
                Auth::Single(basic_provider_1.clone()),
                Auth::Single(basic_provider_2.clone()),
                Auth::Single(jwt_provider.clone())
            ])
        );

        assert_eq!(
            Auth::Single(jwt_provider.clone()).and(Auth::All(vec![
                Auth::Single(basic_provider_1.clone()),
                Auth::Single(basic_provider_2.clone())
            ])),
            Auth::All(vec![
                Auth::Single(basic_provider_1.clone()),
                Auth::Single(basic_provider_2.clone()),
                Auth::Single(jwt_provider.clone()),
            ])
        );

        assert_eq!(
            Auth::Any(vec![
                Auth::Single(jwt_provider.clone()),
                Auth::Single(jwt_provider.clone())
            ])
            .and(Auth::Any(vec![
                Auth::Single(basic_provider_1.clone()),
                Auth::Single(basic_provider_2.clone())
            ])),
            Auth::All(vec![
                Auth::Any(vec![
                    Auth::Single(jwt_provider.clone()),
                    Auth::Single(jwt_provider.clone())
                ]),
                Auth::Any(vec![
                    Auth::Single(basic_provider_1.clone()),
                    Auth::Single(basic_provider_2.clone())
                ])
            ])
        );
    }

    #[test]
    fn test_or() {
        let basic_provider_1 = AuthProvider::Basic(BasicProvider { htpasswd: "1".into() });
        let basic_provider_2 = AuthProvider::Basic(BasicProvider { htpasswd: "2".into() });
        let jwt_provider = AuthProvider::Jwt(JwtProvider::test_value());

        assert_eq!(
            Auth::Single(basic_provider_1.clone()).or(Auth::Single(basic_provider_2.clone())),
            Auth::Any(vec![
                Auth::Single(basic_provider_1.clone()),
                Auth::Single(basic_provider_2.clone())
            ])
        );

        assert_eq!(
            Auth::Any(vec![
                Auth::Single(basic_provider_1.clone()),
                Auth::Single(basic_provider_2.clone())
            ])
            .or(Auth::Single(jwt_provider.clone())),
            Auth::Any(vec![
                Auth::Single(basic_provider_1.clone()),
                Auth::Single(basic_provider_2.clone()),
                Auth::Single(jwt_provider.clone())
            ])
        );

        assert_eq!(
            Auth::Single(jwt_provider.clone()).or(Auth::Any(vec![
                Auth::Single(basic_provider_1.clone()),
                Auth::Single(basic_provider_2.clone())
            ])),
            Auth::Any(vec![
                Auth::Single(basic_provider_1.clone()),
                Auth::Single(basic_provider_2.clone()),
                Auth::Single(jwt_provider.clone()),
            ])
        );

        assert_eq!(
            Auth::All(vec![
                Auth::Single(jwt_provider.clone()),
                Auth::Single(jwt_provider.clone())
            ])
            .or(Auth::All(vec![
                Auth::Single(basic_provider_1.clone()),
                Auth::Single(basic_provider_2.clone())
            ])),
            Auth::Any(vec![
                Auth::All(vec![
                    Auth::Single(jwt_provider.clone()),
                    Auth::Single(jwt_provider.clone())
                ]),
                Auth::All(vec![
                    Auth::Single(basic_provider_1.clone()),
                    Auth::Single(basic_provider_2.clone())
                ])
            ])
        );
    }
}
