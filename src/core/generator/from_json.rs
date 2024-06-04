use serde_json::Value;
use url::Url;

use super::json::{
    ConfigPipeline, FieldBaseUrlGenerator, QueryGenerator, SchemaGenerator, TypesGenerator,
};
use crate::core::config::transformer::{Transform, TypeMerger};
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

fn has_same_domain(config_gen_req: &[ConfigGenerationRequest]) -> bool {
    let mut has_same_domain = true;
    let mut url_domain = "";
    for cfg_gen_req in config_gen_req.iter() {
        let domain = cfg_gen_req.url.host_str().unwrap_or_default();
        if domain.is_empty() || !url_domain.is_empty() && url_domain != domain {
            has_same_domain = false;
            break;
        }
        url_domain = domain;
    }
    has_same_domain
}

pub fn from_json(
    config_gen_req: &[ConfigGenerationRequest],
    query: &str,
) -> anyhow::Result<Config> {
    let mut type_counter = 1;

    let url_for_schema = match has_same_domain(config_gen_req) {
        true => Some(config_gen_req[0].url.clone()),
        false => None,
    };

    let mut step_config_gen = ConfigPipeline::default();

    for (i, request) in config_gen_req.iter().enumerate() {
        let operation_field_name = format!("f{}", i + 1);
        let query_operation_gen = QueryGenerator::new(
            request.resp.is_array(),
            &request.url,
            query,
            &operation_field_name,
        );
        step_config_gen = step_config_gen.then(TypesGenerator::new(
            &request.resp,
            &mut type_counter,
            query_operation_gen,
        ));

        if url_for_schema.is_none() {
            // if all API's are of not same domain, then add base url in each field of query
            // opeartion.
            step_config_gen = step_config_gen.then(FieldBaseUrlGenerator::new(&request.url, query))
        }
    }

    step_config_gen = step_config_gen.then(SchemaGenerator::new(
        Some(query.to_string()),
        url_for_schema,
    ));

    let mut config = step_config_gen.get()?;

    let unused_types = config.unused_types();
    config = config.remove_types(unused_types);

    config = TypeMerger::default().transform(config).to_result()?;

    Ok(config)
}
