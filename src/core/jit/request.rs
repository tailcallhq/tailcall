use std::collections::HashMap;

use async_graphql::parser::types::OperationType;

use super::{Builder, Error, ExecutionPlan, Result};
use crate::core::blueprint::Blueprint;

pub struct Request<Value> {
    pub operation: String,
    pub operation_type: OperationType,
    pub variables: HashMap<String, Value>,
    pub extensions: HashMap<String, Value>,
}

impl<Value> Request<Value> {
    pub fn try_plan_from(&self, blueprint: Blueprint) -> Result<ExecutionPlan> {
        let doc = async_graphql::parser::parse_query(&self.operation)?;
        let builder = Builder::new(blueprint, doc);
        builder.build().map_err(Error::BuildError)
    }
}
