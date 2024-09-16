use crate::core::{jit::OperationPlan, valid::Valid};

use super::Rule;

pub struct QueryComplexity {
    depth: usize,
}

impl QueryComplexity {
    pub fn new(depth: usize) -> Self {
        Self { depth }
    }
}

impl Rule for QueryComplexity {
    fn validate(&self, plan: &OperationPlan<async_graphql_value::Value>) -> Valid<(), String> {
        todo!()
    }
}