use serde::{Deserialize, Serialize};

use crate::is_default;
#[derive(Clone, Debug, Eq, Serialize, Deserialize, PartialEq, schemars::JsonSchema)]
/// The `groupBy` parameter groups multiple data requests into a single call. For more details please refer out [n + 1 guide](https://tailcall.run/docs/guides/n+1#solving-using-batching).
pub struct GroupBy {
    #[serde(default, skip_serializing_if = "is_default")]
    path: Vec<String>,
}

impl GroupBy {
    pub fn new(path: Vec<String>) -> Self {
        Self { path }
    }

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
        Self { path: vec![ID.to_string()] }
    }
}
