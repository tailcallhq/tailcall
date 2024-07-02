use std::collections::HashMap;

use async_graphql_value::{ConstValue, Name, Variables};
use derive_setters::Setters;
use indexmap::IndexMap;
use serde::Deserialize;

use super::{Builder, Error, ExecutionPlan, Result};
use crate::core::blueprint::Blueprint;

#[derive(Debug, Deserialize, Setters)]
pub struct Request<Value> {
    pub query: String,
    pub operation_name: Option<String>,
    pub variables: HashMap<String, Value>,
    pub extensions: HashMap<String, Value>,
}

impl<Value> Request<Value> {
    pub fn try_new(&self, blueprint: &Blueprint) -> Result<ExecutionPlan>
    where
        Value: Clone + Into<ConstValue>,
    {
        let doc = async_graphql::parser::parse_query(&self.query)?;
        let variables: IndexMap<Name, ConstValue> = self
            .variables
            .iter()
            .map(|(k, v)| (Name::new(k), v.clone().into()))
            .collect();
        let variables = Variables::from_value(ConstValue::Object(variables));
        let builder = Builder::new(blueprint, doc, Some(variables));
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
        }
    }
}
