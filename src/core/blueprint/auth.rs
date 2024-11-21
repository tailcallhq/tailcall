use std::collections::{BTreeMap, HashSet};
use std::fmt::Debug;

use jsonwebtoken::jwk::JwkSet;
use tailcall_valid::Valid;

use crate::core::config::ConfigModule;

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

impl Provider {
    /// Used to collect all auth providers from the config module
    pub fn from_config_module(
        config_module: &ConfigModule,
    ) -> Valid<BTreeMap<String, Provider>, String> {
        let mut providers = BTreeMap::new();

        // Add basic auth providers from htpasswd
        for htpasswd in &config_module.extensions().htpasswd {
            if let Some(id) = &htpasswd.id {
                providers.insert(
                    id.clone(),
                    Provider::Basic(Basic { htpasswd: htpasswd.content.clone() }),
                );
            }
        }

        // Add JWT providers from jwks
        for jwks in &config_module.extensions().jwks {
            if let Some(id) = &jwks.id {
                providers.insert(
                    id.clone(),
                    Provider::Jwt(Jwt {
                        jwks: jwks.content.clone(),
                        issuer: None,
                        audiences: HashSet::new(),
                        optional_kid: false,
                    }),
                );
            }
        }

        Valid::succeed(providers)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Auth {
    Provider(Provider),
    And(Vec<Auth>),
    Or(Vec<Auth>),
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
        Auth::And(vec![self, other])
    }

    pub fn or(self, other: Self) -> Self {
        Auth::Or(vec![self, other])
    }

    pub fn simplify(&self) -> Self {
        match self {
            Self::And(expressions) => {
                // Recursively simplify all expressions
                let simplified_exprs: Vec<Self> =
                    expressions.iter().map(|e| e.simplify()).collect();

                // Flatten any nested And expressions
                let mut final_exprs = Vec::new();
                let mut or_exprs_to_remove = Vec::new();

                // Collect the conditions from the And expressions
                for expr in simplified_exprs {
                    match expr {
                        Self::And(inner) => {
                            // Flatten inner And expressions by adding their contents
                            final_exprs.extend(inner);
                        }
                        Self::Or(ref or_exprs) => {
                            // If the OR expression has C1, check if we already have C1 in the AND
                            if let Some(Self::Provider(ref cond1)) = or_exprs.first() {
                                if final_exprs.iter().any(|e| {
                                    if let Self::Provider(ref cond2) = e {
                                        cond2 == cond1
                                    } else {
                                        false
                                    }
                                }) {
                                    // If we already have C1 in the AND, we can skip this OR
                                    // expression
                                    or_exprs_to_remove.push(expr.clone());
                                }
                            }
                            // Always add the Or expressions to the final list (unless already
                            // removed)
                            final_exprs.push(expr);
                        }
                        _ => final_exprs.push(expr),
                    }
                }

                // Remove any redundant OR expressions
                final_exprs.retain(|expr| !or_exprs_to_remove.contains(expr));

                // Remove duplicate conditions
                let mut unique_exprs = Vec::new();
                for expr in final_exprs {
                    if !unique_exprs.contains(&expr) {
                        unique_exprs.push(expr);
                    }
                }

                // Return the simplified AND expression
                Self::And(unique_exprs)
            }
            Self::Or(expressions) => {
                // Recursively simplify all expressions
                let simplified_exprs: Vec<Self> =
                    expressions.iter().map(|e| e.simplify()).collect();

                // Flatten any nested Or expressions
                let mut final_exprs = Vec::new();
                let mut and_exprs_to_remove = Vec::new();

                // Collect the conditions from the Or expressions
                for expr in simplified_exprs {
                    match expr {
                        Self::Or(inner) => {
                            // Flatten inner Or expressions by adding their contents
                            final_exprs.extend(inner);
                        }
                        Self::And(ref and_exprs) => {
                            // If the And expression has C1, check if we already have C1 in the Or
                            if let Some(Self::Provider(ref cond1)) = and_exprs.first() {
                                if final_exprs.iter().any(|e| {
                                    if let Self::Provider(ref cond2) = e {
                                        cond2 == cond1
                                    } else {
                                        false
                                    }
                                }) {
                                    // If we already have C1 in the Or, we can skip this And
                                    // expression
                                    and_exprs_to_remove.push(expr.clone());
                                }
                            }
                            // Always add the And expressions to the final list (unless already
                            // removed)
                            final_exprs.push(expr);
                        }
                        _ => final_exprs.push(expr),
                    }
                }

                // Remove any redundant And expressions
                final_exprs.retain(|expr| !and_exprs_to_remove.contains(expr));

                // Remove duplicate conditions
                let mut unique_exprs = Vec::new();
                for expr in final_exprs {
                    if !unique_exprs.contains(&expr) {
                        unique_exprs.push(expr);
                    }
                }

                // Return the simplified Or expression
                Self::Or(unique_exprs)
            }
            Self::Provider(_) => {
                // If it's a condition, return it as is
                self.clone()
            }
        }
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
            Auth::And(vec![
                Auth::Provider(basic_provider_1),
                Auth::Provider(basic_provider_2),
            ])
        )
    }

    #[test]
    fn and_basic_with_jwt() {
        let basic_provider = test_basic_provider_1();
        let jwt_provider = test_jwt_provider();

        assert_eq!(
            Auth::Provider(basic_provider.clone()).and(Auth::Provider(jwt_provider.clone())),
            Auth::And(vec![
                Auth::Provider(basic_provider),
                Auth::Provider(jwt_provider),
            ])
        );
    }

    #[test]
    fn and_nested_and_with_jwt() {
        let basic_provider_1 = test_basic_provider_1();
        let basic_provider_2 = test_basic_provider_2();
        let jwt_provider = test_jwt_provider();

        assert_eq!(
            Auth::And(vec![
                Auth::Provider(basic_provider_1.clone()),
                Auth::Provider(basic_provider_2.clone())
            ])
            .and(Auth::Provider(jwt_provider.clone())),
            Auth::And(vec![
                Auth::And(vec![
                    Auth::Provider(basic_provider_1),
                    Auth::Provider(basic_provider_2)
                ]),
                Auth::Provider(jwt_provider)
            ])
        );
    }

    #[test]
    fn simplify_and_same_providers() {
        let basic_provider = Provider::Basic(Basic { htpasswd: "1".into() });

        let auth =
            Auth::Provider(basic_provider.clone()).and(Auth::Provider(basic_provider.clone()));

        assert_eq!(
            auth.simplify(),
            Auth::And(vec![Auth::Provider(basic_provider)])
        );
    }

    #[test]
    fn simplify_or_same_providers() {
        let basic_provider = Provider::Basic(Basic { htpasswd: "1".into() });

        let auth =
            Auth::Provider(basic_provider.clone()).or(Auth::Provider(basic_provider.clone()));

        assert_eq!(
            auth.simplify(),
            Auth::Or(vec![Auth::Provider(basic_provider)])
        );
    }

    #[test]
    fn simplify_nested_case_1() {
        let basic_provider_1 = Provider::Basic(Basic { htpasswd: "1".into() });
        let basic_provider_2 = Provider::Basic(Basic { htpasswd: "2".into() });
        let basic_provider_3 = Provider::Basic(Basic { htpasswd: "3".into() });

        let auth = Auth::And(vec![
            Auth::And(vec![
                Auth::Provider(basic_provider_1.clone()),
                Auth::Provider(basic_provider_2.clone()),
            ]),
            Auth::Or(vec![
                Auth::Provider(basic_provider_1.clone()),
                Auth::Provider(basic_provider_3.clone()),
            ]),
        ]);

        let expected = Auth::And(vec![
            Auth::Provider(basic_provider_1),
            Auth::Provider(basic_provider_2),
        ]);

        assert_eq!(auth.simplify().simplify(), expected);
    }

    #[test]
    fn simplify_nested_case_2() {
        let basic_provider_1 = Provider::Basic(Basic { htpasswd: "1".into() });
        let basic_provider_2 = Provider::Basic(Basic { htpasswd: "2".into() });
        let basic_provider_3 = Provider::Basic(Basic { htpasswd: "3".into() });
        let basic_provider_4 = Provider::Basic(Basic { htpasswd: "4".into() });
        let basic_provider_5 = Provider::Basic(Basic { htpasswd: "5".into() });

        let auth = Auth::And(vec![
            Auth::And(vec![
                Auth::Provider(basic_provider_1.clone()),
                Auth::Provider(basic_provider_2.clone()),
            ]),
            Auth::Or(vec![
                Auth::Provider(basic_provider_1.clone()),
                Auth::Provider(basic_provider_3.clone()),
            ]),
            Auth::Or(vec![
                Auth::Provider(basic_provider_4.clone()),
                Auth::Provider(basic_provider_5.clone()),
            ]),
        ]);

        let expected = Auth::And(vec![
            Auth::Provider(basic_provider_1),
            Auth::Provider(basic_provider_2),
            Auth::Or(vec![
                Auth::Provider(basic_provider_4.clone()),
                Auth::Provider(basic_provider_5.clone()),
            ]),
        ]);

        assert_eq!(auth.simplify(), expected);
    }

    #[test]
    fn simplify_nested_case_3() {
        let basic_provider_1 = Provider::Basic(Basic { htpasswd: "1".into() });
        let basic_provider_2 = Provider::Basic(Basic { htpasswd: "2".into() });
        let basic_provider_3 = Provider::Basic(Basic { htpasswd: "3".into() });
        let basic_provider_4 = Provider::Basic(Basic { htpasswd: "4".into() });

        let auth = Auth::And(vec![
            Auth::And(vec![
                Auth::Provider(basic_provider_1.clone()),
                Auth::Provider(basic_provider_2.clone()),
            ]),
            Auth::Or(vec![
                Auth::Provider(basic_provider_1.clone()),
                Auth::Provider(basic_provider_3.clone()),
            ]),
            Auth::Provider(basic_provider_4.clone()),
        ]);

        let expected = Auth::And(vec![
            Auth::Provider(basic_provider_1),
            Auth::Provider(basic_provider_2),
            Auth::Provider(basic_provider_4.clone()),
        ]);

        assert_eq!(auth.simplify(), expected);
    }

    #[test]
    fn simplify_nested_case_4() {
        let basic_provider_1 = Provider::Basic(Basic { htpasswd: "1".into() });
        let basic_provider_2 = Provider::Basic(Basic { htpasswd: "2".into() });
        let basic_provider_3 = Provider::Basic(Basic { htpasswd: "3".into() });
        let basic_provider_4 = Provider::Basic(Basic { htpasswd: "4".into() });
        let basic_provider_5 = Provider::Basic(Basic { htpasswd: "5".into() });

        let basic_provider_6 = Provider::Basic(Basic { htpasswd: "6".into() });
        let basic_provider_7 = Provider::Basic(Basic { htpasswd: "7".into() });
        let basic_provider_8 = Provider::Basic(Basic { htpasswd: "8".into() });
        let basic_provider_9 = Provider::Basic(Basic { htpasswd: "9".into() });
        let basic_provider_10 = Provider::Basic(Basic { htpasswd: "10".into() });

        let auth = Auth::And(vec![
            Auth::And(vec![
                Auth::And(vec![
                    Auth::Provider(basic_provider_1.clone()),
                    Auth::Provider(basic_provider_2.clone()),
                ]),
                Auth::Or(vec![
                    Auth::Provider(basic_provider_1.clone()),
                    Auth::Provider(basic_provider_3.clone()),
                ]),
                Auth::Or(vec![
                    Auth::Provider(basic_provider_4.clone()),
                    Auth::Provider(basic_provider_5.clone()),
                ]),
            ]),
            Auth::And(vec![
                Auth::And(vec![
                    Auth::Provider(basic_provider_1.clone()),
                    Auth::Provider(basic_provider_2.clone()),
                ]),
                Auth::Or(vec![
                    Auth::Provider(basic_provider_1.clone()),
                    Auth::Provider(basic_provider_3.clone()),
                ]),
                Auth::Or(vec![
                    Auth::Provider(basic_provider_4.clone()),
                    Auth::Provider(basic_provider_5.clone()),
                ]),
            ]),
            Auth::And(vec![
                Auth::And(vec![
                    Auth::Provider(basic_provider_6.clone()),
                    Auth::Provider(basic_provider_7.clone()),
                ]),
                Auth::Or(vec![
                    Auth::Provider(basic_provider_6.clone()),
                    Auth::Provider(basic_provider_8.clone()),
                ]),
                Auth::Or(vec![
                    Auth::Provider(basic_provider_9.clone()),
                    Auth::Provider(basic_provider_10.clone()),
                ]),
            ]),
        ]);

        let expected = Auth::And(vec![
            Auth::Provider(basic_provider_1),
            Auth::Provider(basic_provider_2),
            Auth::Or(vec![
                Auth::Provider(basic_provider_4.clone()),
                Auth::Provider(basic_provider_5.clone()),
            ]),
            Auth::Provider(basic_provider_6),
            Auth::Provider(basic_provider_7),
            Auth::Or(vec![
                Auth::Provider(basic_provider_9.clone()),
                Auth::Provider(basic_provider_10.clone()),
            ]),
        ]);

        assert_eq!(auth.simplify(), expected);
    }

    #[test]
    fn simplify_no_change() {
        let basic_provider_1 = Provider::Basic(Basic { htpasswd: "1".into() });
        let basic_provider_2 = Provider::Basic(Basic { htpasswd: "2".into() });

        let auth = Auth::And(vec![
            Auth::Provider(basic_provider_1.clone()),
            Auth::Provider(basic_provider_2.clone()),
        ]);

        assert_eq!(auth.clone().simplify(), auth);
    }
}
