use serde::{Deserialize, Serialize};

use crate::auth::jwt::JwtProviderOptions;

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, Eq)]
pub struct Auth {
  #[serde(rename = "JWT")]
  pub(crate) jwt: Option<JwtProviderOptions>,
}

impl Auth {
  pub fn merge_right(self, other: Auth) -> Self {
    Self { jwt: self.jwt.or(other.jwt) }
  }
}
