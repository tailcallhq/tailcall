use crate::core::config::Config;
use crate::core::generator::json::ConfigGenerator;

pub struct SchemaGenerator {
    query_name: Option<String>,
}

impl SchemaGenerator {
    pub fn new(query_name: Option<String>) -> Self {
        Self { query_name }
    }
}

impl ConfigGenerator for SchemaGenerator {
    fn apply(&mut self, mut config: Config) -> Config {
        config.schema.query = self.query_name.clone();
        // TODO: add support for subscriptions and mutation later on.
        config
    }
}

#[cfg(test)]
mod test {
    use crate::core::generator::json::ConfigGenerator;

    use super::SchemaGenerator;
    
    #[test]
    fn test_schema_generator_with_query() {
        let mut schema_gen = SchemaGenerator::new(Some("Query".to_string()));
        let config = schema_gen.apply(Default::default());
        insta::assert_snapshot!(config.to_sdl())
    }

    #[test]
    fn test_schema_generator_without_query() {
        let mut schema_gen = SchemaGenerator::new(None);
        let config = schema_gen.apply(Default::default());
        assert!(config.to_sdl().is_empty());
    }
}
