use serde_json::Value;
use url::Url;

use super::json::{
    FieldBaseUrlGenerator, NameGenerator, QueryGenerator, SchemaGenerator, TypesGenerator,
};
use crate::core::config::transformer::{RemoveUnused, Transform, TransformerOps, TypeMerger};
use crate::core::config::Config;
use crate::core::valid::Validator;

pub struct ConfigGenerationRequest {
    url: Url,
    resp: Value,
}

impl ConfigGenerationRequest {
    pub fn new(url: Url, resp: Value) -> Self {
        Self { url, resp }
    }
}

pub fn from_json(
    config_gen_req: &[ConfigGenerationRequest],
    query: &str,
) -> anyhow::Result<Config> {
    let mut config = Config::default();
    let field_name_gen = NameGenerator::new("f");
    let type_name_gen = NameGenerator::new("T");

    for request in config_gen_req.iter() {
        let field_name = field_name_gen.generate_name();
        let query_generator =
            QueryGenerator::new(request.resp.is_array(), &request.url, query, &field_name);

        config = TypesGenerator::new(&request.resp, query_generator, &type_name_gen)
            .pipe(SchemaGenerator::new(query.to_owned()))
            .pipe(FieldBaseUrlGenerator::new(&request.url, query))
            .pipe(RemoveUnused)
            .pipe(TypeMerger::new(0.8)) //TODO: take threshold value from user
            .transform(config)
            .to_result()?;
    }

    Ok(config)
}
