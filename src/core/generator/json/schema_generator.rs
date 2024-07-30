use crate::core::config::Config;
use crate::core::transform::Transform;
use crate::core::valid::Valid;

pub struct SchemaGenerator<'a> {
    query_name: &'a Option<String>,
    mutation_name: &'a Option<String>,
}

impl<'a> SchemaGenerator<'a> {
    pub fn new(query_name: &'a Option<String>, mutation_name: &'a Option<String>) -> Self {
        Self { query_name, mutation_name }
    }
}

impl Transform for SchemaGenerator<'_> {
    type Value = Config;
    type Error = String;
    fn transform(&self, mut config: Self::Value) -> Valid<Self::Value, Self::Error> {
        if self.query_name.is_none() && self.mutation_name.is_none() {
            return Valid::fail(
                "Error: At least one of Query or Mutation type must be present in the schema."
                    .to_owned(),
            );
        }
        if let Some(q_name) = self.query_name {
            config.schema.query = Some(q_name.to_owned());
        }
        if let Some(mutation_name) = self.mutation_name {
            config.schema.mutation = Some(mutation_name.to_owned());
        }
        Valid::succeed(config)
    }
}

#[cfg(test)]
mod test {
    use super::SchemaGenerator;
    use crate::core::transform::Transform;
    use crate::core::valid::Validator;

    #[test]
    fn test_schema_generator() {
        let query_name = Some("Query".into());
        let mutation_name = Some("Mutation".into());
        let schema_gen = SchemaGenerator::new(&query_name, &mutation_name);
        let config = schema_gen
            .transform(Default::default())
            .to_result()
            .unwrap();
        insta::assert_snapshot!(config.to_sdl());
    }

    #[test]
    fn test_should_raise_error_if_operation_type_are_missing() {
        let schema_gen = SchemaGenerator::new(&None, &None);
        let result = schema_gen.transform(Default::default()).to_result();
        assert!(result.is_err());
    }
}
