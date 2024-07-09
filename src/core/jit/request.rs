use std::collections::HashMap;

use async_graphql::parser::types::ExecutableDocument;
use derive_setters::Setters;
use serde::Deserialize;

use super::{Builder, Error, ExecutionPlan, Result};
use crate::core::blueprint::Blueprint;

#[derive(Debug, Deserialize, Setters)]
pub struct Request<Value> {
    #[serde(default)]
    pub query: String,
    #[serde(default, rename = "operationName")]
    pub operation_name: Option<String>,
    #[serde(default)]
    pub variables: HashMap<String, Value>,
    #[serde(default)]
    pub extensions: HashMap<String, Value>,
    #[serde(skip)]
    pub document: Option<ExecutableDocument>,
}

impl From<async_graphql::Request> for Request<async_graphql_value::ConstValue> {
    fn from(mut value: async_graphql::Request) -> Self {
        let executable_doc = value.parsed_query().ok().cloned();
        Self {
            query: value.query,
            operation_name: value.operation_name,
            variables: match value.variables.into_value() {
                async_graphql_value::ConstValue::Object(val) => {
                    HashMap::from_iter(val.into_iter().map(|(k, v)| (k.to_string(), v)))
                }
                _ => HashMap::new(),
            },
            extensions: value.extensions,
            document: executable_doc,
        }
    }
}

impl<Value> Request<Value> {
    pub fn try_new(&self, blueprint: &Blueprint) -> Result<ExecutionPlan> {
        let doc = async_graphql::parser::parse_query(&self.query)?;
        let builder = Builder::new(blueprint, doc);
        builder.build().map_err(Error::BuildError)
    }
}

impl<A> Request<A> {
    pub fn new(query: &str) -> Self {
        Self {
            query: query.to_string(),
            operation_name: None,
            variables: HashMap::new(),
            extensions: HashMap::new(),
            document: async_graphql::parser::parse_query(query).ok(),
        }
    }
}
