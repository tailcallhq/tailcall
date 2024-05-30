use serde_json::Value;
use url::Url;

use super::json::{QueryGenerator, SchemaGenerator, TypesGenerator};
use crate::core::config::Config;
use crate::core::generator::json::StepConfigGenerator;
use crate::core::merge_right::MergeRight;

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
    let mut type_counter = 1;
    for (i, request) in config_gen_req.iter().enumerate() {
        let operation_field_name = format!("f{}", i + 1);
        let query_operation_gen = QueryGenerator::new(
            request.resp.is_array(),
            &request.url,
            query,
            &operation_field_name,
        );
        let generated_config = StepConfigGenerator::default()
            .pipe(TypesGenerator::new(
                &request.resp,
                &mut type_counter,
                query_operation_gen,
            ))
            .pipe(SchemaGenerator::new(Some(query.to_string()), None))
            .get();

        config = config.merge_right(generated_config);
    }

    let unused_types = config.unused_types();
    config = config.remove_types(unused_types);

    Ok(config)
}
