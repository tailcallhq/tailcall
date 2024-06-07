use async_graphql_value::ConstValue;
use schemars::schema::Schema;
use schemars::{schema_for, JsonSchema};

/// Represents signed integer type 32bit size
#[derive(JsonSchema, Default)]
pub struct Int32(pub i32);

impl super::Scalar for Int32 {
    fn validate(&self) -> fn(&ConstValue) -> bool {
        |value| {
            if let ConstValue::Number(n) = value {
                if let Some(n) = n.as_i64() {
                    return i32::try_from(n).is_ok();
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

    use super::Int32;
    use crate::core::scalar::Scalar;
    use crate::{test_scalar_invalid, test_scalar_valid};

    test_scalar_valid! {
        Int32,
        ConstValue::Number(Number::from(100u32)),
        ConstValue::Number(Number::from(i32::MAX as i64)),
        ConstValue::Number(
            Number::from(-15)
        )
    }

    test_scalar_invalid! {
        Int32,
        ConstValue::Null,
        ConstValue::Number(Number::from(i32::MAX as i64 + 1)),
        ConstValue::Number(Number::from(i32::MIN as i64 - 1)),
        ConstValue::Number(
            Number::from_f64(1.25).unwrap()
        ),
        ConstValue::String("4564846".to_string())
    }
}
