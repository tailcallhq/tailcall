use serde::{Deserialize, Serialize};

use crate::core::is_default;
#[derive(Clone, Debug, Eq, Serialize, Deserialize, PartialEq, schemars::JsonSchema)]
/// The `groupBy` parameter groups multiple data requests into a single call. For more details please refer out [n + 1 guide](https://tailcall.run/docs/guides/n+1#solving-using-batching).
#[serde(rename_all = "camelCase")]
pub struct GroupBy {
    #[serde(default, skip_serializing_if = "is_default")]
    batch_key: String,
    #[serde(default, skip_serializing_if = "is_default")]
    extraction_path: Vec<String>,
}

impl GroupBy {
    pub fn new(batch_key: String, extraction_path: Vec<String>) -> Self {
        Self { batch_key, extraction_path }
    }

    pub fn path(&self) -> Vec<String> {
        if self.extraction_path.is_empty() {
            return vec![String::from(ID)];
        }
        self.extraction_path.clone()
    }

    pub fn key(&self) -> &str {
        self.batch_key.as_str()
    }
}

const ID: &str = "id";

impl Default for GroupBy {
    fn default() -> Self {
        Self {
            batch_key: ID.to_string(),
            extraction_path: vec![ID.to_string()],
        }
    }
}
