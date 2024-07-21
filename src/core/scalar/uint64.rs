use schemars::schema::Schema;
use schemars::{schema_for, JsonSchema};
use tailcall_macros::ScalarDefinition;

use crate::core::json::{JsonLike, JsonLikeOwned};

/// Represents unsigned integer type 64bit size as string
#[derive(JsonSchema, Default, ScalarDefinition)]
pub struct UInt64(pub u64);

impl super::Scalar for UInt64 {
    fn validate_owned<Value: JsonLikeOwned>(&self) -> fn(&Value) -> bool {
        |value| value.as_str().map_or(false, |s| s.parse::<u64>().is_ok())
    }

    fn validate<'a, Value: JsonLike<'a>>(&self) -> fn(&'a Value) -> bool {
        |value| value.as_str().map_or(false, |s| s.parse::<u64>().is_ok())
    }

    fn schema(&self) -> Schema {
        schema_for!(Self).schema.into()
    }
}

#[cfg(test)]
mod test {
    use async_graphql_value::ConstValue;
    use serde_json::Number;

    use super::UInt64;
    use crate::core::scalar::Scalar;
    use crate::{test_scalar_invalid, test_scalar_valid};

    test_scalar_valid! {
        UInt64,
        ConstValue::String("125".to_string()),
        ConstValue::String(u64::MAX.to_string())
    }

    test_scalar_invalid! {
        UInt64,
        ConstValue::Null,
        ConstValue::Number(Number::from(15)),
        ConstValue::Number(
            Number::from_f64(1.25).unwrap()
        ),
        ConstValue::String("-1".to_string()),
        ConstValue::String(format!("{}1", u64::MAX))
    }
}
