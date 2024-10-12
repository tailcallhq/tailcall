use std::collections::BTreeSet;

use convert_case::{Case, Casing};
use tailcall_valid::Valid;

use crate::core::config::{Config, GraphQLOperationType};
use crate::core::transform::Transform;

pub struct SchemaGenerator<'a> {
    operation_type: &'a GraphQLOperationType,
    header_keys: &'a Option<BTreeSet<String>>,
}

impl<'a> SchemaGenerator<'a> {
    pub fn new(
        operation_type: &'a GraphQLOperationType,
        header_keys: &'a Option<BTreeSet<String>>,
    ) -> Self {
        Self { operation_type, header_keys }
    }
}

impl Transform for SchemaGenerator<'_> {
    type Value = Config;
    type Error = String;
    fn transform(&self, mut config: Self::Value) -> Valid<Self::Value, Self::Error> {
        match self.operation_type {
            GraphQLOperationType::Query => {
                config.schema.query = Some(
                    GraphQLOperationType::Query
                        .to_string()
                        .to_case(Case::Pascal),
                );
            }
            GraphQLOperationType::Mutation => {
                config.schema.mutation = Some(
                    GraphQLOperationType::Mutation
                        .to_string()
                        .to_case(Case::Pascal),
                );
            }
        }

        // Add allowed headers setting on upstream
        config.upstream = config.upstream.allowed_headers(self.header_keys.to_owned());

        Valid::succeed(config)
    }
}

#[cfg(test)]
mod test {
    use std::collections::BTreeSet;

    use tailcall_valid::Validator;

    use super::SchemaGenerator;
    use crate::core::config::GraphQLOperationType;
    use crate::core::transform::Transform;

    #[test]
    fn test_schema_generator_with_mutation() {
        let schema_gen = SchemaGenerator::new(&GraphQLOperationType::Mutation, &None);
        let config = schema_gen
            .transform(Default::default())
            .to_result()
            .unwrap();
        assert!(config.schema.mutation.is_some());
        assert_eq!(config.schema.mutation, Some("Mutation".to_owned()));

        assert!(config.schema.query.is_none());
    }

    #[test]
    fn test_schema_generator_with_query() {
        let schema_gen = SchemaGenerator::new(&GraphQLOperationType::Query, &None);
        let config = schema_gen
            .transform(Default::default())
            .to_result()
            .unwrap();
        assert!(config.schema.query.is_some());
        assert_eq!(config.schema.query, Some("Query".to_owned()));

        assert!(config.schema.mutation.is_none());
    }

    #[test]
    fn test_schema_generator_with_headers() {
        let expected_header_keys = Some(BTreeSet::from(["X-Custom-Header".to_owned()]));
        let schema_gen = SchemaGenerator::new(&GraphQLOperationType::Query, &expected_header_keys);
        let config = schema_gen
            .transform(Default::default())
            .to_result()
            .unwrap();
        assert!(config.schema.query.is_some());
        assert_eq!(config.schema.query, Some("Query".to_owned()));
        assert_eq!(config.upstream.allowed_headers, expected_header_keys);

        assert!(config.schema.mutation.is_none());
    }
}
