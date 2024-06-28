use async_graphql_value::ConstValue;
use schemars::schema::Schema;
use schemars::{schema_for, JsonSchema};
use tailcall_macros::DocumentDefinition;

/// The JSON scalar type represents JSON values as specified by
/// [ECMA-404](www.ecma-international.org/publications/files/ECMA-ST/
/// ECMA-404.pdf).
#[derive(JsonSchema, Default, DocumentDefinition)]
#[allow(clippy::upper_case_acronyms)]
#[doc_type("Scalar")]
pub struct JSON {
    #[allow(dead_code)]
    #[serde(rename = "JSON")]
    pub json: String, // we don't care about the type, this is just for documentation
}

impl super::Scalar for JSON {
    fn validate(&self) -> fn(&ConstValue) -> bool {
        |_| true
    }

    fn schema(&self) -> Schema {
        Schema::Object(schema_for!(Self).schema)
    }
}
