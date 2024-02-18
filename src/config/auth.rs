use std::collections::HashSet;
use std::num::NonZeroU64;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::is_default;
use crate::mustache::Mustache;
use crate::runtime::TargetRuntimeContext;

mod default {
    pub mod jwt {
        pub mod remote {
            use std::num::NonZeroU64;

            pub fn max_age() -> NonZeroU64 {
                NonZeroU64::new(5 * 60 * 1000).unwrap()
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Basic {
    pub htpasswd: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Jwks {
    /// JWKS data as a string or a template
    Data(String),
    /// JWKS data loaded from the remote server
    #[serde(rename_all = "camelCase")]
    Remote {
        url: String,
        #[serde(default = "default::jwt::remote::max_age")]
        max_age: NonZeroU64,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Jwt {
    #[serde(skip_serializing_if = "is_default")]
    pub issuer: Option<String>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub audiences: HashSet<String>,
    /// Specifies if the kid value inside request's JWT token is required to get validated with the JWKS
    #[serde(default, skip_serializing_if = "is_default")]
    pub optional_kid: bool,
    /// Specifies JWKS data that is used for JWT validation.
    /// More on [jwks](https://datatracker.ietf.org/doc/html/rfc7517).
    /// If you need to create JWKS from private key use tools like [this](https://russelldavies.github.io/jwk-creator/)
    pub jwks: Jwks,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum AuthProvider {
    /// Settings for JWT auth provider
    Jwt(Jwt),
    /// Settings for Basic auth provider
    Basic(Basic),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, schemars::JsonSchema)]
pub struct AuthEntry {
    /// Unique id for the auth provider. For future use
    pub id: Option<String>,
    #[serde(flatten)]
    pub provider: AuthProvider,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, Eq, schemars::JsonSchema)]
pub struct Auth(pub Vec<AuthEntry>);

impl Auth {
    pub fn merge_right(self, other: Auth) -> Self {
        let mut providers = self.0;

        providers.extend(other.0);

        Self(providers)
    }

    pub fn is_some(&self) -> bool {
        !self.0.is_empty()
    }

    pub fn render_mustache(&mut self, runtime_ctx: &TargetRuntimeContext) -> Result<()> {
        for entry in self.0.iter_mut() {
            match &mut entry.provider {
                AuthProvider::Jwt(jwt) => match &mut jwt.jwks {
                    Jwks::Data(ref mut jwks) => {
                        let tmpl = Mustache::parse(jwks)?;

                        *jwks = tmpl.render(runtime_ctx);
                    }
                    Jwks::Remote { ref mut url, .. } => {
                        let tmpl = Mustache::parse(url)?;

                        *url = tmpl.render(runtime_ctx);
                    }
                },
                AuthProvider::Basic(Basic { ref mut htpasswd }) => {
                    let tmpl = Mustache::parse(htpasswd)?;

                    *htpasswd = tmpl.render(runtime_ctx);
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use anyhow::Result;
    use serde_json::json;

    use super::*;

    #[test]
    fn jwt_options_parse() -> Result<()> {
        let config: Jwt = serde_json::from_value(json!({
          "optionalKid": true,
          "jwks": {
            "remote": {
              "url": "http://localhost:3000"
            }
          }
        }))?;

        assert!(matches!(
            config,
            Jwt { optional_kid: true, jwks: Jwks::Remote { .. }, .. }
        ));

        Ok(())
    }
}
