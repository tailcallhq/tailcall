use schemars::JsonSchema;
use tailcall_macros::ScalarDefinition;

use crate::core::json::JsonLike;

/// Represents signed integer type 8bit size
#[derive(JsonSchema, Default, ScalarDefinition, Clone, Debug)]
pub struct Int8(pub i8);

impl super::Scalar for Int8 {
    fn validate<'a, Value: JsonLike<'a>>(&self) -> fn(&'a Value) -> bool {
        |value| {
            if let Some(value) = value.as_i64() {
                return i8::try_from(value).is_ok();
            }
            false
        }
    }
}

#[cfg(test)]
mod test {
    use async_graphql_value::ConstValue;
    use serde_json::Number;

    use super::Int8;
    use crate::core::scalar::Scalar;
    use crate::{test_scalar_invalid, test_scalar_valid};

    test_scalar_valid! {
        Int8,
        ConstValue::Number(Number::from(100i32)),
        ConstValue::Number(Number::from(127)),
        ConstValue::Number(
            Number::from(-15)
        )
    }

    test_scalar_invalid! {
        Int8,
        ConstValue::Null,
        ConstValue::Number(Number::from(128)),
        ConstValue::Number(Number::from(-129)),
        ConstValue::Number(
            Number::from_f64(1.25).unwrap()
        ),
        ConstValue::String("4564846".to_string())
    }
}
