use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GroupBy {
  key: String,
  path: Option<Vec<String>>,
}

const EMPTY_VEC: &Vec<String> = &vec![];

impl GroupBy {
  pub fn path(&self) -> &Vec<String> {
    self.path.as_ref().unwrap_or(EMPTY_VEC)
  }

  pub fn key(&self) -> &String {
    &self.key
  }

  pub fn new(key: String, path: Vec<String>) -> GroupBy {
    GroupBy { key, path: Some(path) }
  }
}
