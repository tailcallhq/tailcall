use anyhow::Context;
use serde_json::Value;
use url::Url;

use super::json::{
    FieldBaseUrlGenerator, NameGenerator, QueryGenerator, SchemaGenerator, TypesGenerator,
};
use super::Generate;
use crate::core::config::transformer::{
    ConsolidateURL, Pipe, RemoveUnused, Transform, Transformer, TransformerOps, TypeMerger,
    TypeNameGenerator,
};
use crate::core::config::Config;
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

pub struct FromJson<'a> {
    request_samples: Option<Vec<RequestSample>>,
    type_name_generator: &'a NameGenerator,
    field_name_generator: &'a NameGenerator,
    operation_name: Option<String>,
    is_mutation: Option<bool>,
}

impl<'a> FromJson<'a> {
    pub fn new(type_name_generator: &'a NameGenerator, field_name_gen: &'a NameGenerator) -> Self {
        FromJson {
            request_samples: None,
            type_name_generator: &type_name_generator,
            field_name_generator: &field_name_gen,
            operation_name: None,
            is_mutation: None,
        }
    }

    pub fn with_samples(mut self, request_samples: Vec<RequestSample>) -> Self {
        self.request_samples = Some(request_samples);
        self
    }

    pub fn with_operation(mut self, operation_name: String) -> Self {
        self.operation_name = Some(operation_name);
        self
    }

    pub fn with_mutation(mut self, is_mutation: bool) -> Self {
        self.is_mutation = Some(is_mutation);
        self
    }

    pub fn generate(self) -> anyhow::Result<Config> {
        let request_samples = self
            .request_samples
            .context("request samples are required in order to generate the config.")?;
        let operation_name = self
            .operation_name
            .context("operation name is required to generate the config.")?;

        let config_generator = FromJsonGenerator {
            request_samples,
            type_name_generator: self.type_name_generator,
            field_name_generator: self.field_name_generator,
            operation_name,
        };

        let config = config_generator.generate().to_result()?;
        Ok(config)
    }
}

struct FromJsonGenerator<'a> {
    request_samples: Vec<RequestSample>,
    type_name_generator: &'a NameGenerator,
    field_name_generator: &'a NameGenerator,
    operation_name: String,
}

impl Generate for FromJsonGenerator<'_> {
    type Error = String;

    fn generate(&self) -> Valid<Config, Self::Error> {
        let config_gen_req = &self.request_samples;
        let field_name_gen = self.field_name_generator;
        let type_name_gen = self.type_name_generator;
        let query = &self.operation_name;

        let mut config_pipeline: Option<Pipe<_, _>> = None;
        for sample in config_gen_req {
            let field_name = field_name_gen.generate_name();
            let query_generator =
                QueryGenerator::new(sample.response.is_array(), &sample.url, query, &field_name);

            // these transformations are required in order to generate a base config.
            let transforms = TypesGenerator::new(&sample.response, query_generator, type_name_gen)
                .pipe(SchemaGenerator::new(query.to_owned()))
                .pipe(FieldBaseUrlGenerator::new(&sample.url, query))
                .pipe(RemoveUnused)
                .pipe(TypeMerger::default())
                .pipe(TypeNameGenerator);

            config_pipeline = Some(Transformer::pipe(transforms, Transformer::empty()));
        }

        if let Some(pipe_line) = config_pipeline {
            return pipe_line.transform(Default::default());
        }

        Valid::fail("config generation pipeline failed".to_string())
    }
}

pub fn from_json(
    config_gen_req: &[RequestSample],
    query: &str,
    field_name_gen: &NameGenerator,
    type_name_gen: &NameGenerator,
) -> anyhow::Result<Config> {
    let mut config = Config::default();

    for request in config_gen_req.iter() {
        let field_name = field_name_gen.generate_name();
        let query_generator = QueryGenerator::new(
            request.response.is_array(),
            &request.url,
            query,
            &field_name,
        );

        config = TypesGenerator::new(&request.response, query_generator, type_name_gen)
            .pipe(SchemaGenerator::new(query.to_owned()))
            .pipe(FieldBaseUrlGenerator::new(&request.url, query))
            .pipe(RemoveUnused)
            .pipe(TypeMerger::new(0.8)) //TODO: take threshold value from user
            .pipe(TypeNameGenerator)
            .transform(config)
            .to_result()?;
    }

    let config = ConsolidateURL::new(0.5).transform(config).to_result()?;

    Ok(config)
}
