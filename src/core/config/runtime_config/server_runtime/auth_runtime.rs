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
    use once_cell::sync::Lazy;

    use super::*;

    pub static JWK_SET: Lazy<JwkSet> = Lazy::new(|| {
        let value = serde_json::json!({
          "keys": [
            {
              "kty": "RSA",
              "use": "sig",
              "alg": "RS256",
              "kid": "I48qMJp566SSKQogYXYtHBo9q6ZcEKHixNPeNoxV1c8",
              "n": "ksMb5oMlhJ_HzAebCuBG6-v5Qc4J111ur7Aux6-8SbxzqFONsf2Bw6ATG8pAfNeZ-USA3_T1mGkYTDvfoggXnxsduWV_lePZKKOq_Qp_EDdzic1bVTJQDad3CXldR3wV6UFDtMx6cCLXxPZM5n76e7ybPt0iNgwoGpJE28emMZJXrnEUFzxwFMq61UlzWEumYqW3uOUVp7r5XAF5jQ_1nQAnpHBnRFzdNPVb3E6odMGu3jgp8mkPbPMP16Fund4LVplLz8yrsE9TdVrSdYJThylRWn_BwvJ0DjUcp8ibJya86iClUlixAmBwR9NdStHwQqHwmMXMKkTXo-ytRmSUobzxX9T8ESkij6iBhQpmDMD3FbkK30Y7pUVEBBOyDfNcWOhholjOj9CRrxu9to5rc2wvufe24VlbKb9wngS_uGfK4AYvVyrcjdYMFkdqw-Mft14HwzdO2BTS0TeMDZuLmYhj_bu5_g2Zu6PH5OpIXF6Fi8_679pCG8wWAcFQrFrM0eA70wD_SqD_BXn6pWRpFXlcRy_7PWTZ3QmC7ycQFR6Wc6Px44y1xDUoq3rH0RlZkeicfvP6FRlpjFU7xF6LjAfd9ciYBZfJll6PE7zf-i_ZXEslv-tJ5-30-I4Slwj0tDrZ2Z54OgAg07AIwAiI5o4y-0vmuhUscNpfZsGAGhE",
              "e": "AQAB"
            },
            {
              "kty": "RSA",
              "n": "u1SU1LfVLPHCozMxH2Mo4lgOEePzNm0tRgeLezV6ffAt0gunVTLw7onLRnrq0_IzW7yWR7QkrmBL7jTKEn5u-qKhbwKfBstIs-bMY2Zkp18gnTxKLxoS2tFczGkPLPgizskuemMghRniWaoLcyehkd3qqGElvW_VDL5AaWTg0nLVkjRo9z-40RQzuVaE8AkAFmxZzow3x-VJYKdjykkJ0iT9wCS0DRTXu269V264Vf_3jvredZiKRkgwlL9xNAwxXFg0x_XFw005UWVRIkdgcKWTjpBP2dPwVZ4WWC-9aGVd-Gyn1o0CLelf4rEjGoXbAAEgAqeGUxrcIlbjXfbcmw",
              "e": "AQAB",
              "alg": "RS256"
            }
          ]
        });

        serde_json::from_value(value).unwrap()
    });

    impl JwtRuntime {
        fn test_value() -> Self {
            Self {
                issuer: Default::default(),
                audiences: Default::default(),
                optional_kid: false,
                jwks: JWK_SET.clone(),
            }
        }
    }

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
