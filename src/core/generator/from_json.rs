use serde_json::Value;
use url::Url;

use super::json::{
    FieldBaseUrlGenerator, NameGenerator, QueryGenerator, SchemaGenerator, TypesGenerator,
};
use super::Generate;
use crate::core::config::transformer::{
    ConsolidateURL, RemoveUnused, Transform, TransformerOps, TypeMerger, TypeNameGenerator,
};
use crate::core::config::Config;
use crate::core::merge_right::MergeRight;
use crate::core::valid::{Valid, Validator};

pub struct RequestSample {
    url: Url,
    response: Value,
}

impl RequestSample {
    pub fn new(url: Url, resp: Value) -> Self {
        Self { url, response: resp }
    }
}

pub struct FromJsonGenerator<'a> {
    request_samples: Vec<RequestSample>,
    type_name_generator: &'a NameGenerator,
    field_name_generator: &'a NameGenerator,
    operation_name: String,
}

impl<'a> FromJsonGenerator<'a> {
    pub fn new(
        request_samples: Vec<RequestSample>,
        type_name_generator: &'a NameGenerator,
        field_name_generator: &'a NameGenerator,
        operation_name: &str,
    ) -> Self {
        Self {
            request_samples,
            type_name_generator,
            field_name_generator,
            operation_name: operation_name.to_string(),
        }
    }
}

impl Generate for FromJsonGenerator<'_> {
    type Error = String;

    fn generate(&self) -> Valid<Config, Self::Error> {
        let config_gen_req = &self.request_samples;
        let field_name_gen = self.field_name_generator;
        let type_name_gen = self.type_name_generator;
        let query = &self.operation_name;

        let mut config = Config::default();

        for sample in config_gen_req {
            let field_name = field_name_gen.generate_name();
            let query_generator =
                QueryGenerator::new(sample.response.is_array(), &sample.url, query, &field_name);

            // these transformations are required in order to generate a base config.
            let transform_pipeline =
                TypesGenerator::new(&sample.response, query_generator, type_name_gen)
                    .pipe(SchemaGenerator::new(query.to_owned()))
                    .pipe(FieldBaseUrlGenerator::new(&sample.url, query))
                    .pipe(RemoveUnused)
                    .pipe(TypeMerger::default())
                    .pipe(TypeNameGenerator);

            if let Ok(generated_config) = transform_pipeline.transform(config.clone()).to_result() {
                config = config.merge_right(generated_config);
            }
        }

        ConsolidateURL::new(0.5).transform(config)
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use crate::core::generator::{FromJsonGenerator, Generate, NameGenerator, RequestSample};
    use crate::core::valid::Validator;

    #[derive(Deserialize)]
    struct JsonFixture {
        url: String,
        body: serde_json::Value,
    }

    fn parse_json(path: &str) -> JsonFixture {
        let content = std::fs::read_to_string(path).unwrap();
        serde_json::from_str(&content).unwrap()
    }

    #[test]
    fn generate_config_from_json() -> anyhow::Result<()> {
        let mut request_samples = vec![];
        let fixtures = [
            "src/core/generator/tests/fixtures/json/incompatible_properties.json",
            "src/core/generator/tests/fixtures/json/list_incompatible_object.json",
            "src/core/generator/tests/fixtures/json/nested_list.json",
            "src/core/generator/tests/fixtures/json/nested_same_properties.json",
            "src/core/generator/tests/fixtures/json/incompatible_root_object.json",
        ];
        for fixture in fixtures {
            let parsed_content = parse_json(fixture);
            request_samples.push(RequestSample {
                url: parsed_content.url.parse()?,
                response: parsed_content.body,
            });
        }

        let config = FromJsonGenerator::new(
            request_samples,
            &NameGenerator::new("T"),
            &NameGenerator::new("f"),
            "Query",
        )
        .generate()
        .to_result()?;

        insta::assert_snapshot!(config.to_sdl());
        Ok(())
    }
}
