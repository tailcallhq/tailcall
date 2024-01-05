use std::collections::HashSet;
use std::num::NonZeroU64;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::is_default;

mod remote {
  use std::num::NonZeroU64;

  pub fn default_max_age() -> NonZeroU64 {
    NonZeroU64::new(5 * 60 * 1000).unwrap()
  }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum JwksVerifierOptions {
  Const(Value),
  File(PathBuf),
  #[serde(rename_all = "camelCase")]
  Remote {
    // TODO: could be Url, but parsing error in that case is misleading
    // `Parsing failed because of invalid value: string \"__unknown.json\", expected relative URL without a base`
    url: String,
    #[serde(default = "remote::default_max_age")]
    max_age: NonZeroU64,
  },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct JwksOptions {
  #[serde(default, skip_serializing_if = "is_default")]
  pub optional_kid: bool,
  #[serde(flatten)]
  pub verifier: JwksVerifierOptions,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct JwtProviderOptions {
  #[serde(skip_serializing_if = "is_default")]
  pub issuer: Option<String>,
  #[serde(default, skip_serializing_if = "is_default")]
  pub audiences: HashSet<String>,
  pub jwks: JwksOptions,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum AuthProviderConfig {
  JWT(JwtProviderOptions),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct AuthConfig {
  pub id: String,
  #[serde(flatten)]
  pub provider: AuthProviderConfig,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, Eq)]
pub struct Auth(pub Vec<AuthConfig>);

impl Auth {
  pub fn merge_right(self, other: Auth) -> Self {
    let mut providers = self.0;

    providers.extend(other.0);

    Self(providers)
  }

  pub fn is_some(&self) -> bool {
    !self.0.is_empty()
  }
}
