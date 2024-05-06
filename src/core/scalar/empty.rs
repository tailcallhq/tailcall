use async_graphql_value::ConstValue;
use schemars::schema::Schema;
use schemars::{schema_for, JsonSchema};

#[derive(JsonSchema, Default)]
pub struct Empty {
    #[serde(rename = "Empty")]
    /// Empty scalar type represents an empty value.
    pub empty: (), // we don't care about the type, this is just for documentation
}

impl super::Scalar for Empty {
    fn validate(&self) -> fn(&ConstValue) -> bool {
        |_| true
    }

    fn scalar(&self) -> Schema {
        Schema::Object(schema_for!(Self).schema)
    }
}
