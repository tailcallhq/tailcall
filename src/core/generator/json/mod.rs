mod field_base_url_generator;
mod http_directive_generator;
mod query_generator;
mod schema_generator;
mod type_name_generator;
mod types_generator;
mod url_utils;

use std::cell::RefCell;

pub use field_base_url_generator::FieldBaseUrlGenerator;
pub use query_generator::QueryGenerator;
pub use schema_generator::SchemaGenerator;
pub use type_name_generator::TypeNameGenerator;
pub use types_generator::TypesGenerator;

use crate::core::counter::Counter;

pub struct NameGenerator {
    counter: Counter,
    prefix: String,
    current_name: RefCell<String>,
}

impl NameGenerator {
    pub fn new(prefix: &str) -> Self {
        Self {
            counter: Counter::new(1),
            prefix: prefix.to_string(),
            current_name: RefCell::new(format!("{}{}", prefix, 1)),
        }
    }

    pub fn get_name(&self) -> String {
        self.current_name.borrow().to_owned()
    }

    pub fn generate_name(&self) -> String {
        let id = self.counter.next();
        let generated_name = format!("{}{}", self.prefix, id);
        self.current_name.replace(generated_name.to_owned());
        generated_name
    }
}
