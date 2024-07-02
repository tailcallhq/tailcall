use serde_json::Value;
use url::Url;

use super::json::{self, TypesGenerator};
use super::NameGenerator;
use crate::core::config::Config;
use crate::core::merge_right::MergeRight;
use crate::core::transform::{Transform, TransformerOps};
use crate::core::valid::{Valid, Validator};

pub struct RequestSample {
    url: Url,
    response: Value,
    field_name: String,
}

impl RequestSample {
    pub fn new(url: Url, resp: Value, field_name: &str) -> Self {
        Self { url, response: resp, field_name: field_name.to_string() }
    }
}

pub struct FromJsonGenerator<'a> {
    request_samples: &'a [RequestSample],
    type_name_generator: &'a NameGenerator,
    operation_name: String,
}

impl<'a> FromJsonGenerator<'a> {
    pub fn new(
        request_samples: &'a [RequestSample],
        type_name_generator: &'a NameGenerator,
        operation_name: &str,
    ) -> Self {
        Self {
            request_samples,
            type_name_generator,
            operation_name: operation_name.to_string(),
        }
    }
}

impl Transform for FromJsonGenerator<'_> {
    type Value = Config;
    type Error = String;
    fn transform(&self, config: Self::Value) -> Valid<Self::Value, Self::Error> {
        let config_gen_req = self.request_samples;
        let type_name_gen = self.type_name_generator;
        let query = &self.operation_name;

        Valid::from_iter(config_gen_req, |sample| {
            let field_name = &sample.field_name;
            let query_generator = json::QueryGenerator::new(
                sample.response.is_array(),
                &sample.url,
                query,
                field_name,
            );

            // these transformations are required in order to generate a base config.
            TypesGenerator::new(&sample.response, query_generator, type_name_gen)
                .pipe(json::SchemaGenerator::new(query.to_owned()))
                .pipe(json::FieldBaseUrlGenerator::new(&sample.url, query))
                .transform(config.clone())
        })
        .map(|configs| {
            configs
                .iter()
                .fold(config, |acc, c| acc.merge_right(c.clone()))
        })
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use crate::core::config::transformer::Preset;
    use crate::core::generator::{FromJsonGenerator, NameGenerator, RequestSample};
    use crate::core::transform::TransformerOps;
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
        let field_name_generator = NameGenerator::new("f");
        for fixture in fixtures {
            let parsed_content = parse_json(fixture);
            request_samples.push(RequestSample::new(
                parsed_content.url.parse()?,
                parsed_content.body,
                &field_name_generator.next(),
            ));
        }

        let config = FromJsonGenerator::new(&request_samples, &NameGenerator::new("T"), "Query")
            .pipe(Preset::default())
            .generate()
            .to_result()?;

        insta::assert_snapshot!(config.to_sdl());
        Ok(())
    }
}
