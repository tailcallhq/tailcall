use super::Rule;
use crate::core::jit::{Field, Nested, OperationPlan};
use crate::core::valid::Valid;

pub struct QueryDepth(usize);

impl QueryDepth {
    pub fn new(depth: usize) -> Self {
        Self(depth)
    }
}

impl Rule for QueryDepth {
    fn validate(&self, plan: &OperationPlan<async_graphql_value::Value>) -> Valid<(), String> {
        let depth = plan
            .as_nested()
            .iter()
            .map(|field| Self::depth_helper(field, 1))
            .max()
            .unwrap_or(0);

        if depth > self.0 {
            Valid::fail("Query Depth validation failed.".into())
        } else {
            Valid::succeed(())
        }
    }
}

impl QueryDepth {
    /// Helper function to recursively calculate depth.
    fn depth_helper(
        field: &Field<Nested<async_graphql_value::Value>, async_graphql_value::Value>,
        current_depth: usize,
    ) -> usize {
        let mut max_depth = current_depth;

        if let Some(child) = field.extensions.as_ref() {
            for nested_child in child.0.iter() {
                let depth = Self::depth_helper(nested_child, current_depth + 1);
                if depth > max_depth {
                    max_depth = depth;
                }
            }
        }
        max_depth
    }
}
