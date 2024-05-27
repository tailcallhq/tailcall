use async_graphql_value::ConstValue;
use schemars::schema::Schema;
use schemars::{schema_for, JsonSchema};

/// Represents unsigned integer type 64 bit size
#[derive(JsonSchema, Default)]
pub struct Int64(pub i64);

impl super::Scalar for Int64 {
    fn validate(&self) -> fn(&ConstValue) -> bool {
        |value| {
            if let ConstValue::Number(n) = value {
                n.as_i64().is_some()
            } else {
                false
            }
        }
    }
    fn schema(&self) -> Schema {
        schema_for!(Self).schema.into()
    }
}

#[cfg(test)]
mod test {
    use async_graphql_value::ConstValue;
    use serde_json::Number;

    use crate::core::scalar::{Int64, Scalar};
    use crate::{test_scalar_invalid, test_scalar_valid};

    test_scalar_valid! {
        Int64,
        ConstValue::Number(Number::from(100u32)),
        ConstValue::Number(Number::from(2 * u32::MAX as u64)),
        ConstValue::Number(
            Number::from(-15)
        )
    }

    test_scalar_invalid! {
        Int64,
        ConstValue::Null,
        ConstValue::Number(
            Number::from_f64(1.25).unwrap()
        ),
        ConstValue::String("4564846".to_string())
    }
}
