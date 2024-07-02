use std::collections::HashMap;

use async_graphql_value::ConstValue;
use derive_setters::Setters;
use serde::Deserialize;

use super::{ConstBuilder, Error, ExecutionPlan, Result};
use crate::core::blueprint::Blueprint;
use crate::core::jit::builder::Builder;

#[derive(Debug, Deserialize, Setters)]
pub struct Request<Value> {
    pub query: String,
    pub operation_name: Option<String>,
    pub variables: HashMap<String, Value>,
    pub extensions: HashMap<String, Value>,
}

impl Request<ConstValue> {
    pub fn try_new(
        &self,
        blueprint: &Blueprint,
    ) -> Result<ExecutionPlan<async_graphql_value::Value, async_graphql::Value>> {
        let doc = async_graphql::parser::parse_query(&self.query)?;
        let builder = ConstBuilder::new(blueprint, doc, Some(self.variables.clone()));
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
