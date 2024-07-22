use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::core::is_default;
#[derive(Clone, Debug, Eq, Serialize, Deserialize, PartialEq, schemars::JsonSchema)]
/// The `groupBy` parameter groups multiple data requests into a single call. For more details please refer out [n + 1 guide](https://tailcall.run/docs/guides/n+1#solving-using-batching).
pub struct GroupBy {
    #[serde(default, skip_serializing_if = "is_default")]
    path: Vec<GroupByEnum>,
}

#[derive(Clone, Debug, Eq, Serialize, Deserialize, PartialEq, schemars::JsonSchema)]
pub enum GroupByEnum {
    #[serde(rename = "query")]
    Query(String),
    #[serde(rename = "rename")]
    Rename { query: String, object: String },
}

impl GroupByEnum {
    pub fn as_str(&self) -> &str {
        match self {
            GroupByEnum::Query(value) => value.as_str(),
            GroupByEnum::Rename { query, object: _ } => query.as_str(),
        }
    }
}

impl Display for GroupByEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}

impl GroupBy {
    pub fn new(path: Vec<serde_json::Value>) -> Self {
        Self {
            path: path
                .into_iter()
                .map(|path| match path {
                    serde_json::Value::String(value) => GroupByEnum::Query(value),
                    serde_json::Value::Object(object) => GroupByEnum::Rename {
                        query: object.get("query").unwrap().to_string().replace("\"", ""),
                        object: object.get("object").unwrap().to_string().replace("\"", ""),
                    },
                    _ => panic!("Invalid batchKey value: {}", path),
                })
                .collect(),
        }
    }

    pub fn path(&self) -> Vec<String> {
        if self.path.is_empty() {
            return vec![String::from(ID)];
        }
        self.path
            .clone()
            .into_iter()
            .map(|value| match value {
                GroupByEnum::Rename { query, object: _ } => query,
                GroupByEnum::Query(value) => value,
            })
            .collect()
    }

    pub fn path_target(&self) -> Vec<String> {
        if self.path.is_empty() {
            return vec![String::from(ID)];
        }
        self.path
            .clone()
            .into_iter()
            .map(|value| match value {
                GroupByEnum::Rename { query: _, object } => object,
                GroupByEnum::Query(value) => value,
            })
            .collect()
    }

    pub fn key(&self) -> &str {
        self.path
            .last()
            .map(|a| match a {
                GroupByEnum::Rename { query, object: _ } => query.as_str(),
                GroupByEnum::Query(value) => value.as_str(),
            })
            .unwrap_or(ID)
    }
}

const ID: &str = "id";

impl Default for GroupBy {
    fn default() -> Self {
        Self { path: vec![GroupByEnum::Query(ID.to_string())] }
    }
}
