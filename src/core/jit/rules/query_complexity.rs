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
    fn validate<T: std::fmt::Debug>(&self, plan: &OperationPlan<T>) -> Valid<(), String> {
        let complexity: usize = plan.as_nested().iter().map(Self::complexity_helper).sum();
        if complexity > self.0 {
            Valid::fail("Query is too complex.".into())
        } else {
            Valid::succeed(())
        }
    }
}

impl QueryComplexity {
    fn complexity_helper<T: std::fmt::Debug>(field: &Field<Nested<T>, T>) -> usize {
        let mut complexity = 1;

        let fields = field.iter_only(|_| true).collect::<Vec<_>>();
        for child in fields {
            complexity += Self::complexity_helper(child);
        }

        complexity
    }
}

#[cfg(test)]
mod test {
    use async_graphql_value::ConstValue;

    use crate::core::blueprint::Blueprint;
    use crate::core::config::Config;
    use crate::core::jit::rules::{QueryComplexity, Rule};
    use crate::core::jit::{Builder, OperationPlan, Variables};
    use crate::core::valid::Validator;

    const CONFIG: &str = include_str!("./../fixtures/jsonplaceholder-mutation.graphql");

    fn plan(query: impl AsRef<str>) -> OperationPlan<ConstValue> {
        let config = Config::from_sdl(CONFIG).to_result().unwrap();
        let blueprint = Blueprint::try_from(&config.into()).unwrap();
        let document = async_graphql::parser::parse_query(query).unwrap();
        let variables: Variables<ConstValue> = Variables::default();

        Builder::new(&blueprint, document)
            .build(&variables, None)
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

        let plan = plan(query);
        let query_complexity = QueryComplexity::new(4);
        let val_result = query_complexity.validate(&plan);
        assert!(val_result.is_succeed());

        let query_complexity = QueryComplexity::new(2);
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

        let plan = plan(query);

        let query_complexity = QueryComplexity::new(6);
        let val_result = query_complexity.validate(&plan);
        assert!(val_result.is_succeed());

        let query_complexity = QueryComplexity::new(5);
        let val_result = query_complexity.validate(&plan);
        assert!(!val_result.is_succeed());
    }
}
