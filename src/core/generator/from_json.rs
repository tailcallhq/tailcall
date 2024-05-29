use serde_json::Value;
use url::Url;

use crate::core::config::Config;
use crate::core::generator::json::schema_generator::SchemaGenerator;
use crate::core::generator::json::upstream_generator::UpstreamGenerator;
use crate::core::generator::json::StepConfigGenerator;
use crate::core::merge_right::MergeRight;

use super::json::types_generator::TypesGenerator;

pub struct ConfigGenerationRequest<'a> {
    url: &'a str,
    resp: &'a Value,
}

impl<'a> ConfigGenerationRequest<'a> {
    pub fn new(url: &'a str, resp: &'a Value) -> Self {
        Self { url, resp }
    }
}

pub fn from_json(config_gen_req: &[ConfigGenerationRequest]) -> anyhow::Result<Config> {
    let mut config = Config::default();
    let query = "Query";
    let mut type_counter = 1;
    for (i, request) in config_gen_req.iter().enumerate() {
        let url = Url::parse(request.url).unwrap();
        let generated_config = StepConfigGenerator::default()
            .pipe(TypesGenerator::new(
                request.resp.clone(),
                &mut type_counter,
                url.clone(),
                format!("f{}", i + 1),
                query.to_string(),
            ))
            .pipe(UpstreamGenerator::new(url))
            .pipe(SchemaGenerator::new(Some(query.to_string())))
            .generate();
        
        config = config.merge_right(generated_config);
    }

    let unused_types = config.unused_types();
    config = config.remove_types(unused_types);

    Ok(config)
}
