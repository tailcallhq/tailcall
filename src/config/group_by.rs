use serde::{Deserialize, Serialize};

use crate::config::is_default;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct GroupBy {
  #[serde(default, skip_serializing_if = "is_default")]
  path: Vec<String>,
}

impl GroupBy {
  pub fn path(&self) -> &Vec<String> {
    &self.path
  }

  pub fn key(&self) -> &String {
    self.path.last().unwrap()
  }
}
