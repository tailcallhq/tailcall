use crate::core::{jit::OperationPlan, valid::Valid};

use super::Rule;

pub struct QueryDepth {
    depth: usize,
}

impl QueryDepth {
    pub fn new(depth: usize) -> Self {
        Self { depth }
    }
}

impl Rule for QueryDepth {
    fn validate(&self, plan: &OperationPlan<async_graphql_value::Value>) -> Valid<(), String> {
        todo!()
    }
}