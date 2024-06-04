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

pub trait NameGenerator {
    fn generate_name(&mut self) -> String;
}

pub struct TypeNameGenerator(pub u32);

impl NameGenerator for TypeNameGenerator {
    fn generate_name(&mut self) -> String {
        let generated_name = format!("T{}", self.0);
        self.0 += 1;
        generated_name
    }
}

pub struct FieldNameGenerator(pub u32);

impl NameGenerator for FieldNameGenerator {
    fn generate_name(&mut self) -> String {
        let generated_name = format!("f{}", self.0);
        self.0 += 1;
        generated_name
    }
}
