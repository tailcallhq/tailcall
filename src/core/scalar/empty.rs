use schemars::JsonSchema;
use tailcall_macros::ScalarDefinition;

use crate::core::json::{JsonLike, JsonLikeOwned};

/// Empty scalar type represents an empty value.
#[derive(JsonSchema, Default, ScalarDefinition)]
pub struct Empty {
    #[allow(dead_code)]
    #[serde(rename = "Empty")]
    pub empty: (), // we don't care about the type, this is just for documentation
}

impl super::Scalar for Empty {
    fn validate_owned<Value: JsonLikeOwned>(&self) -> fn(&Value) -> bool {
        |_| true
    }

    fn validate<'a, Value: JsonLike<'a>>(&self) -> fn(&'a Value) -> bool {
        |_| true
    }
}
