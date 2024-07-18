use async_graphql_value::ConstValue;
use schemars::schema::Schema;
use schemars::{schema_for, JsonSchema};
use tailcall_macros::ScalarDefinition;

use crate::core::json::JsonLikeOwned;

/// Empty scalar type represents an empty value.
#[derive(JsonSchema, Default, ScalarDefinition)]
pub struct Empty {
    #[allow(dead_code)]
    #[serde(rename = "Empty")]
    pub empty: (), // we don't care about the type, this is just for documentation
}

impl super::Scalar for Empty {
    fn validate(&self) -> fn(&ConstValue) -> bool {
        |_| true
    }

    fn validate_generic<Value: JsonLikeOwned>(&self) -> fn(&Value) -> bool {
        |_| true
    }

    fn schema(&self) -> Schema {
        Schema::Object(schema_for!(Self).schema)
    }
}
