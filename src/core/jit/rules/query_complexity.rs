use crate::core::{jit::OperationPlan, valid::Valid};

use super::Rule;

pub struct QueryComplexity(usize);

impl QueryComplexity {
    pub fn new(depth: usize) -> Self {
        Self(depth)
    }
}

impl Rule for QueryComplexity {
    fn validate(&self, plan: &OperationPlan<async_graphql_value::Value>) -> Valid<(), String> {
        let complexity = plan.calculate_complexity();
        if complexity > self.0 {
            Valid::fail("Query Complexity validation failed.".into())
        } else {
            Valid::succeed(())
        }
    }
}