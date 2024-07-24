use schemars::JsonSchema;
use tailcall_macros::ScalarDefinition;

use crate::core::json::JsonLike;

/// Empty scalar type represents an empty value.
#[derive(JsonSchema, Default, ScalarDefinition, Clone, Debug)]
pub struct Empty {
    #[allow(dead_code)]
    #[serde(rename = "Empty")]
    pub empty: (), // we don't care about the type, this is just for documentation
}

impl super::Scalar for Empty {
    fn validate<'a, Value: JsonLike<'a>>(&self) -> fn(&'a Value) -> bool {
        |_| true
    }
}
