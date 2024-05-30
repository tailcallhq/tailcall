mod field_base_url_generator;
mod query_generator;
mod schema_generator;
mod types_generator;

pub use field_base_url_generator::FieldBaseUrlGenerator;
pub use query_generator::QueryGenerator;
pub use schema_generator::SchemaGenerator;
pub use types_generator::TypesGenerator;

use crate::core::config::Config;
use crate::core::merge_right::MergeRight;

pub trait ConfigTransformer {
    fn apply(&mut self, config: Config) -> Config;
}

#[derive(Default)]
pub struct ConfigPipeline {
    config: Config,
}

impl ConfigPipeline {
    pub fn then(mut self, mut other: impl ConfigTransformer) -> Self {
        let update_config = other.apply(self.config.clone());
        self.config = self.config.merge_right(update_config);
        self
    }

    pub fn get(self) -> Config {
        self.config
    }
}
