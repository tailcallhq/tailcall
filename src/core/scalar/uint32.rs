use async_graphql_value::ConstValue;
use schemars::schema::Schema;
use schemars::{schema_for, JsonSchema};

/// Represents unsigned integer type 32bit size
#[derive(JsonSchema, Default)]
pub struct UInt32(pub u32);

impl super::Scalar for UInt32 {
    fn validate(&self) -> fn(&ConstValue) -> bool {
        |value| {
            if let ConstValue::Number(n) = value {
                if let Some(n) = n.as_u64() {
                    return u32::try_from(n).is_ok();
                }
            }

            false
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

    use super::UInt32;
    use crate::core::scalar::Scalar;
    use crate::{test_scalar_invalid, test_scalar_valid};

    test_scalar_valid! {
        UInt32,
        ConstValue::Number(Number::from(100u32)),
        ConstValue::Number(Number::from(u32::MAX as u64))
    }

    test_scalar_invalid! {
        UInt32,
        ConstValue::Null,
        ConstValue::Number(Number::from(u32::MAX as u64 + 1)),
        ConstValue::Number(Number::from(-1)),
        ConstValue::Number(
            Number::from_f64(1.25).unwrap()
        ),
        ConstValue::String("4564846".to_string())
    }
}
