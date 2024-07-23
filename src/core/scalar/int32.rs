use schemars::JsonSchema;
use tailcall_macros::ScalarDefinition;

use crate::core::json::JsonLikeOwned;

/// Represents signed integer type 32bit size
#[derive(JsonSchema, Default, ScalarDefinition)]
pub struct Int32(pub i32);

impl super::Scalar for Int32 {
    fn validate<Value: JsonLikeOwned>(&self) -> fn(&Value) -> bool {
        |value| {
            if let Some(n) = value.as_i64() {
                return i32::try_from(n).is_ok();
            }

            false
        }
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
