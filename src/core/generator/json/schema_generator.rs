use crate::core::config::transformer::Transform;
use crate::core::config::Config;
use crate::core::valid::Valid;

pub struct SchemaGenerator {
    query_type: String,
}

impl SchemaGenerator {
    pub fn new(query_type: String) -> Self {
        Self { query_type }
    }

    pub fn generate_schema(&self, config: &mut Config) {
        config.schema.query = Some(self.query_type.to_owned());
        // TODO: add support for mutation and subscription.
    }
}

impl Transform for SchemaGenerator {
    fn transform(&self, mut config: Config) -> Valid<Config, String> {
        self.generate_schema(&mut config);
        Valid::succeed(config)
    }
}

#[cfg(test)]
mod test {
    use anyhow::Ok;

    use super::SchemaGenerator;
    use crate::core::config::transformer::Transform;
    use crate::core::valid::Validator;

    #[test]
    fn test_schema_generator_with_query() -> anyhow::Result<()> {
        let schema_gen = SchemaGenerator::new("Query".to_owned());
        let config = schema_gen.transform(Default::default()).to_result()?;
        insta::assert_snapshot!(config.to_sdl());
        Ok(())
    }
}
