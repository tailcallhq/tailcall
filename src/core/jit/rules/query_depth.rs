use super::Rule;
use crate::core::jit::{Field, Nested, OperationPlan};
use crate::core::valid::Valid;

use async_graphql_value::ConstValue;

pub struct QueryDepth(usize);

impl QueryDepth {
    pub fn new(depth: usize) -> Self {
        Self(depth)
    }
}

impl Rule for QueryDepth {
    type Value = ConstValue;
    type Error = String;
    fn validate(&self, plan: &OperationPlan<Self::Value>) -> Valid<(), Self::Error> {
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
    fn depth_helper(field: &Field<Nested<ConstValue>, ConstValue>, current_depth: usize) -> usize {
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

#[cfg(test)]
mod test {
    use async_graphql_value::ConstValue;

    use super::QueryDepth;
    use crate::core::blueprint::Blueprint;
    use crate::core::config::Config;
    use crate::core::jit::rules::Rule;
    use crate::core::jit::{Builder, OperationPlan, Variables};
    use crate::core::valid::Validator;

    const CONFIG: &str = include_str!("./../fixtures/jsonplaceholder-mutation.graphql");

    fn plan(query: impl AsRef<str>, variables: &Variables<ConstValue>) -> OperationPlan<ConstValue> {
        let config = Config::from_sdl(CONFIG).to_result().unwrap();
        let blueprint = Blueprint::try_from(&config.into()).unwrap();
        let document = async_graphql::parser::parse_query(query).unwrap();
        Builder::new(&blueprint, document)
            .build(variables, None)
            .unwrap()
    }

    #[test]
    fn test_query_complexity() {
        let query = r#"
            {
                posts {
                        id
                        userId
                        title
                }
            }
        "#;

        let plan = plan(query, &Default::default());
        let query_complexity = QueryDepth::new(4);
        let val_result = query_complexity.validate(&plan);
        assert!(val_result.is_succeed());

        let query_complexity = QueryDepth::new(1);
        let val_result = query_complexity.validate(&plan);
        assert!(!val_result.is_succeed());
    }

    #[test]
    fn test_nested_query_complexity() {
        let query = r#"
            {
                posts {
                    id
                    title
                    user {
                        id
                        name
                    }
                }
            }
        "#;

        let plan = plan(query, &Default::default());

        let query_complexity = QueryDepth::new(4);
        let val_result = query_complexity.validate(&plan);
        assert!(val_result.is_succeed());

        let query_complexity = QueryDepth::new(2);
        let val_result = query_complexity.validate(&plan);
        assert!(!val_result.is_succeed());
    }
}