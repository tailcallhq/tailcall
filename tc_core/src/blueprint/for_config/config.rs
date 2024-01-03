use std::fmt;
use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum GraphQLOperationType {
  #[default]
  Query,
  Mutation,
}

impl Display for GraphQLOperationType {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.write_str(match self {
      Self::Query => "query",
      Self::Mutation => "mutation",
    })
  }
}
