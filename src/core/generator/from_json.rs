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
use crate::core::valid::Valid;

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
            return pipe_line
                .pipe(ConsolidateURL::new(0.5))
                .transform(Default::default());
        }

        Valid::fail("config generation pipeline failed".to_string())
    }
}
