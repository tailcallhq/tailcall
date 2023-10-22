use serde::{Deserialize, Serialize};

use crate::config::is_default;
#[derive(Clone, Debug, Eq, Serialize, Deserialize, PartialEq)]
pub struct GroupBy {
  #[serde(default, rename = "groupByPath", skip_serializing_if = "is_default")]
  path: Vec<String>,
  #[serde(rename = "selectKey")]
  select_key: String,
}

impl GroupBy {
  pub fn path(&self) -> Vec<String> {
    if self.path.is_empty() {
      return vec![String::from(ID)];
    }
    self.path.clone()
  }

  pub fn key(&self) -> &str {
    self.path.last().map(|a| a.as_str()).unwrap_or(ID)
  }
}

const ID: &str = "id";

impl Default for GroupBy {
  fn default() -> Self {
    Self { path: vec![ID.to_string()], select_key: ID.to_string() }
  }
}
