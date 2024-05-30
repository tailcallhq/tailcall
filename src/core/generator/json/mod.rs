mod field_base_url_generator;
mod query_generator;
mod schema_generator;
mod types_generator;

use anyhow::Result;
pub use field_base_url_generator::FieldBaseUrlGenerator;
pub use query_generator::QueryGenerator;
pub use schema_generator::SchemaGenerator;
pub use types_generator::TypesGenerator;

use crate::core::config::Config;
use crate::core::merge_right::MergeRight;
use crate::core::valid::{Valid, Validator};

pub trait ConfigTransformer {
    fn apply(&mut self, config: Config) -> Valid<Config, String>;
}

pub struct ConfigPipeline {
    config_result: Valid<Config, String>,
}

impl Default for ConfigPipeline {
    fn default() -> Self {
        Self { config_result: Valid::succeed(Default::default()) }
    }
}

impl ConfigPipeline {
    pub fn then(mut self, mut other: impl ConfigTransformer) -> Self {
        self.config_result = self.config_result.and_then(|config| {
            other
                .apply(config.clone())
                .and_then(|updated_config| Valid::succeed(config.merge_right(updated_config)))
        });
        self
    }

    pub fn get(self) -> Result<Config> {
        Ok(self.config_result.to_result()?)
    }
}
