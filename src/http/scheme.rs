use serde::{Deserialize, Serialize};
use std::fmt::Display;
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
