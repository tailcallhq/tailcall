use std::collections::HashMap;

use async_graphql_value::ConstValue;
use derive_setters::Setters;
use serde::Deserialize;

use super::input_resolver::InputResolver;
use super::{Builder, ExecutionPlan, Result};
use crate::core::blueprint::Blueprint;

#[derive(Debug, Deserialize, Setters)]
pub struct Request<V> {
    #[serde(default)]
    pub query: String,
    #[serde(default, rename = "operationName")]
    pub operation_name: Option<String>,
    #[serde(default)]
    pub variables: HashMap<String, V>,
    #[serde(default)]
    pub extensions: HashMap<String, V>,
}

impl From<async_graphql::Request> for Request<ConstValue> {
    fn from(value: async_graphql::Request) -> Self {
        Self {
            query: value.query,
            operation_name: value.operation_name,
            variables: match value.variables.into_value() {
                ConstValue::Object(val) => {
                    HashMap::from_iter(val.into_iter().map(|(k, v)| (k.to_string(), v)))
                }
                _ => HashMap::new(),
            },
            extensions: value.extensions,
        }
    }
}

impl<V> Request<V> {
    pub fn try_new(&self, blueprint: &Blueprint) -> Result<ExecutionPlan<ConstValue>> {
        let doc = async_graphql::parser::parse_query(&self.query)?;
        let builder = Builder::new(blueprint, doc);
        let plan = builder.build()?;
        let input_resolver = InputResolver::new(plan);

        Ok(input_resolver.resolve_input()?)
    }
}

impl<A> Request<A> {
    pub fn new(query: &str) -> Self {
        Self {
            query: query.to_string(),
            operation_name: None,
            variables: HashMap::new(),
            extensions: HashMap::new(),
        }
    }
}
