use super::Rule;
use crate::core::jit::{Field, Nested, OperationPlan};
use crate::core::valid::Valid;

pub struct QueryComplexity(usize);

impl QueryComplexity {
    pub fn new(depth: usize) -> Self {
        Self(depth)
    }
}

impl Rule for QueryComplexity {
    fn validate(&self, plan: &OperationPlan<async_graphql_value::Value>) -> Valid<(), String> {
        let complexity: usize = plan.as_nested().iter().map(Self::complexity_helper).sum();

        if complexity > self.0 {
            Valid::fail("Query Complexity validation failed.".into())
        } else {
            Valid::succeed(())
        }
    }
}

impl QueryComplexity {
    fn complexity_helper(
        field: &Field<Nested<async_graphql_value::Value>, async_graphql_value::Value>,
    ) -> usize {
        let mut complexity = 1;

        for child in field.iter_only(|_| true) {
            complexity += Self::complexity_helper(child);
        }

        complexity
    }
}
