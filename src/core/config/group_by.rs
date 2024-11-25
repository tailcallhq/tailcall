use serde::{Deserialize, Serialize};

use crate::core::is_default;

/// The `groupBy` parameter groups multiple data requests into a single call. For more details please refer out [n + 1 guide](https://tailcall.run/docs/guides/n+1#solving-using-batching).
#[derive(Clone, Debug, Eq, Serialize, Deserialize, PartialEq, schemars::JsonSchema)]
pub struct GroupBy {
    #[serde(default, skip_serializing_if = "is_default")]
    path: Vec<String>,
    #[serde(default, skip_serializing_if = "is_default")]
    key: Option<String>,
}

impl GroupBy {
    pub fn new(path: Vec<String>, key: Option<String>) -> Self {
        Self { path, key }
    }

    pub fn path(&self) -> Vec<String> {
        if self.path.is_empty() {
            return vec![String::from(ID)];
        }
        self.path.clone()
    }

    pub fn key(&self) -> &str {
        match &self.key {
            Some(value) => value,
            None => {
                if self.path.is_empty() {
                    return ID;
                }
                self.path.last().unwrap()
            }
        }
    }
}

const ID: &str = "id";

impl Default for GroupBy {
    fn default() -> Self {
        Self { path: vec![ID.to_string()], key: None }
    }
}
