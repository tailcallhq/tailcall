use convert_case::{Casing, Case};

use crate::core::config::{Config, GraphQLOperationType};
use crate::core::transform::Transform;
use crate::core::valid::Valid;

pub struct SchemaGenerator<'a> {
    operation_type: &'a GraphQLOperationType,
}

impl<'a> SchemaGenerator<'a> {
    pub fn new(operation_type: &'a GraphQLOperationType) -> Self {
        Self { operation_type }
    }
}

impl Transform for SchemaGenerator<'_> {
    type Value = Config;
    type Error = String;
    fn transform(&self, mut config: Self::Value) -> Valid<Self::Value, Self::Error> {
        match self.operation_type {
            GraphQLOperationType::Query => {
                config.schema.query = Some(GraphQLOperationType::Query.to_string().to_case(Case::Pascal));
            }
            GraphQLOperationType::Mutation => {
                config.schema.mutation = Some(GraphQLOperationType::Mutation.to_string());
            }
        }
        Valid::succeed(config)
    }
}

#[cfg(test)]
mod test {
    use super::SchemaGenerator;
    use crate::core::config::GraphQLOperationType;
    use crate::core::transform::Transform;
    use crate::core::valid::Validator;

    #[test]
    fn test_schema_generator_with_mutation() {
        let schema_gen = SchemaGenerator::new(&GraphQLOperationType::Mutation);
        let config = schema_gen
            .transform(Default::default())
            .to_result()
            .unwrap();
        assert!(config.schema.mutation.is_some());
        assert!(config.schema.query.is_none());
    }

    #[test]
    fn test_schema_generator_with_query() {
        let schema_gen = SchemaGenerator::new(&GraphQLOperationType::Query);
        let config = schema_gen
            .transform(Default::default())
            .to_result()
            .unwrap();
        assert!(config.schema.mutation.is_none());
        assert!(config.schema.query.is_some());
    }
}
