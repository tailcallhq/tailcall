mod field_base_url_generator;
mod http_directive_generator;
mod query_generator;
mod schema_generator;
mod types_generator;
mod url_utils;

pub use field_base_url_generator::FieldBaseUrlGenerator;
pub use query_generator::QueryGenerator;
pub use schema_generator::SchemaGenerator;
pub use types_generator::TypesGenerator;

use crate::core::counter::Counter;

pub struct NameGenerator {
    counter: Counter,
    prefix: String,
}

impl NameGenerator {
    pub fn new(prefix: &str) -> Self {
        Self { counter: Counter::new(1), prefix: prefix.to_string() }
    }

    pub fn generate_name(&self) -> String {
        let id = self.counter.next();
        format!("{}{}", self.prefix, id)
    }
}
