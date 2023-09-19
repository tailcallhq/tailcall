use std::fmt::Display;

use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Scheme {
  Http,
  Https,
}

impl Display for Scheme {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Scheme::Http => write!(f, "http"),
      Scheme::Https => write!(f, "https"),
    }
  }
}
