use crate::core::{jit::OperationPlan, valid::Valid};

use super::Rule;

pub struct QueryDepth(usize);

impl QueryDepth {
    pub fn new(depth: usize) -> Self {
        Self(depth)
    }
}

impl Rule for QueryDepth {
    fn validate(&self, plan: &OperationPlan<async_graphql_value::Value>) -> Valid<(), String> {
        let depth = plan.calculate_depth();
        if depth > self.0 {
            Valid::fail("Query Depth validation failed.".into())
        } else {
            Valid::succeed(())
        }
    }
}