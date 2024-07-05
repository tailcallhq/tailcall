use url::Url;

use super::http_directive_generator::HttpDirectiveGenerator;
use super::types_generator::OperationGenerator;
use crate::core::config::{Config, Field, Type};
use crate::core::valid::Valid;

pub struct QueryGenerator<'a> {
    is_json_list: bool,
    url: &'a Url,
    query: &'a str,
    field_name: &'a str,
}

impl<'a> QueryGenerator<'a> {
    pub fn new(is_json_list: bool, url: &'a Url, query: &'a str, field_name: &'a str) -> Self {
        Self { is_json_list, url, query, field_name }
    }
}

impl OperationGenerator for QueryGenerator<'_> {
    fn generate(&self, root_type: &str, mut config: Config) -> Valid<Config, String> {
        let mut field = Field {
            list: self.is_json_list,
            type_of: root_type.to_owned(),
            ..Default::default()
        };

        // generate required http directive.
        let http_directive_gen = HttpDirectiveGenerator::new(self.url);
        field.http = Some(http_directive_gen.generate_http_directive(&mut field));

        // if type is already present, then append the new field to it else create one.
        if let Some(type_) = config.types.get_mut(self.query) {
            type_.fields.insert(self.field_name.to_owned(), field);
        } else {
            let mut ty = Type::default();
            ty.fields.insert(self.field_name.to_owned(), field);
            config.types.insert(self.query.to_owned(), ty);
        }
        Valid::succeed(config)
    }
}

#[cfg(test)]
mod test {
    use url::Url;

    use super::QueryGenerator;
    use crate::core::generator::json::types_generator::OperationGenerator;
    use crate::core::valid::Validator;

    #[test]
    fn test_list_json_query_generator() -> anyhow::Result<()> {
        let url = Url::parse("http://example.com/path").unwrap();
        let query_generator = QueryGenerator::new(true, &url, "Query", "f1");
        let config = query_generator
            .generate("T1", Default::default())
            .to_result()?;
        insta::assert_snapshot!(config.to_sdl());
        Ok(())
    }

    #[test]
    fn test_query_generator() -> anyhow::Result<()> {
        let url = Url::parse("http://example.com/path").unwrap();
        let query_generator = QueryGenerator::new(false, &url, "Query", "f1");
        let config = query_generator
            .generate("T1", Default::default())
            .to_result()?;
        insta::assert_snapshot!(config.to_sdl());
        Ok(())
    }

    #[test]
    fn test_query_generator_with_query_params() -> anyhow::Result<()> {
        let url = Url::parse("http://example.com/path?q=12&is_verified=true").unwrap();
        let query_generator = QueryGenerator::new(false, &url, "Query", "f1");
        let config = query_generator
            .generate("T1", Default::default())
            .to_result()?;
        insta::assert_snapshot!(config.to_sdl());
        Ok(())
    }

    #[test]
    fn test_query_generator_with_path_variables() -> anyhow::Result<()> {
        let url = Url::parse("http://example.com/users/12").unwrap();
        let query_generator = QueryGenerator::new(false, &url, "Query", "f1");
        let config = query_generator
            .generate("T1", Default::default())
            .to_result()?;
        insta::assert_snapshot!(config.to_sdl());
        Ok(())
    }
}
