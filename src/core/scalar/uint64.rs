use async_graphql_value::ConstValue;
use schemars::schema::Schema;
use schemars::{schema_for, JsonSchema};

/// Represents unsigned integer type 64 bit size
#[derive(JsonSchema, Default)]
pub struct UInt64(pub u64);

impl super::Scalar for UInt64 {
    fn validate(&self) -> fn(&ConstValue) -> bool {
        |value| {
            if let ConstValue::Number(n) = value {
                n.as_u64().is_some()
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

    use crate::core::scalar::{Scalar, UInt64};
    use crate::{test_scalar_invalid, test_scalar_valid};

    test_scalar_valid! {
        UInt64,
        ConstValue::Number(Number::from(100u32)),
        ConstValue::Number(Number::from(2 * u32::MAX as u64))
    }

    test_scalar_invalid! {
        UInt64,
        ConstValue::Null,
        ConstValue::Number(
            Number::from_f64(1.25).unwrap()
        ),
        ConstValue::Number(
            Number::from(-15)
        ),
        ConstValue::String("468846854564".to_string())
    }
}
