use schemars::JsonSchema;
use tailcall_macros::ScalarDefinition;

use crate::core::json::{JsonLike, JsonLikeOwned};

/// Represents unsigned integer type 32bit size
#[derive(JsonSchema, Default, ScalarDefinition)]
pub struct UInt32(pub u32);

impl super::Scalar for UInt32 {
    fn validate_owned<Value: JsonLikeOwned>(&self) -> fn(&Value) -> bool {
        |value| value.as_u64().map_or(false, |n| u32::try_from(n).is_ok())
    }

    fn validate<'a, Value: JsonLike<'a>>(&self) -> fn(&'a Value) -> bool {
        |value| value.as_u64().map_or(false, |n| u32::try_from(n).is_ok())
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
