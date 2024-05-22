use async_graphql_value::ConstValue;
use schemars::schema::Schema;
use schemars::{schema_for, JsonSchema};

/// Empty scalar type represents an empty value.
#[derive(JsonSchema, Default)]
pub struct Empty {
    #[allow(dead_code)]
    #[serde(rename = "Empty")]
    pub empty: (), // we don't care about the type, this is just for documentation
}

impl super::Scalar for Empty {
    fn validate(&self) -> fn(&ConstValue) -> bool {
        |_| true
    }

    fn schema(&self) -> Schema {
        Schema::Object(schema_for!(Self).schema)
    }
}
