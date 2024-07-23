use schemars::JsonSchema;
use tailcall_macros::ScalarDefinition;

use crate::core::json::{JsonLike};

/// Represents unsigned integer type 8bit size
#[derive(JsonSchema, Default, ScalarDefinition)]
pub struct UInt8(pub u8);

impl super::Scalar for UInt8 {
    fn validate<'a, Value: JsonLike<'a>>(&self) -> fn(&'a Value) -> bool {
        |value| value.as_u64().map_or(false, |n| u8::try_from(n).is_ok())
    }
}

#[cfg(test)]
mod test {
    use async_graphql_value::ConstValue;
    use serde_json::Number;

    use super::UInt8;
    use crate::core::scalar::Scalar;
    use crate::{test_scalar_invalid, test_scalar_valid};

    test_scalar_valid! {
        UInt8,
        ConstValue::Number(Number::from(15)),
        ConstValue::Number(Number::from(255))
    }

    test_scalar_invalid! {
        UInt8,
        ConstValue::Null,
        ConstValue::Number(Number::from(256)),
        ConstValue::Number(Number::from(-1)),
        ConstValue::Number(
            Number::from_f64(1.25).unwrap()
        ),
        ConstValue::String("4564846".to_string())
    }
}
