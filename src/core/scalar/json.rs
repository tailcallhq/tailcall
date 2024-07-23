use schemars::JsonSchema;
use tailcall_macros::ScalarDefinition;

use crate::core::json::JsonLikeOwned;

/// The JSON scalar type represents JSON values as specified by
/// [ECMA-404](www.ecma-international.org/publications/files/ECMA-ST/
/// ECMA-404.pdf).
#[derive(JsonSchema, Default, ScalarDefinition)]
#[allow(clippy::upper_case_acronyms)]
pub struct JSON {
    #[allow(dead_code)]
    #[serde(rename = "JSON")]
    pub json: String, // we don't care about the type, this is just for documentation
}

impl super::Scalar for JSON {
    fn validate<Value: JsonLikeOwned>(&self) -> fn(&Value) -> bool {
        |_| true
    }
}
