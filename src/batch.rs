use directive_definition_derive::DirectiveDefinition;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, DirectiveDefinition)]
pub struct Batch {
  key: String,
  path: Option<Vec<String>>,
}

const EMPTY_VEC: &Vec<String> = &vec![];

impl Batch {
  pub fn path(&self) -> &Vec<String> {
    self.path.as_ref().unwrap_or(EMPTY_VEC)
  }

  pub fn key(&self) -> &String {
    &self.key
  }

  pub fn new(key: String, path: Vec<String>) -> Batch {
    Batch { key, path: Some(path) }
  }
}
