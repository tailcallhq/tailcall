use async_graphql_value::ConstValue;
use schemars::schema::Schema;
use schemars::{schema_for, JsonSchema};

/// Represents unsigned integer type 64 bit size as string
#[derive(JsonSchema, Default)]
pub struct UInt64Str(pub u64);

impl super::Scalar for UInt64Str {
    fn validate(&self) -> fn(&ConstValue) -> bool {
        |value| {
            if let ConstValue::String(n) = value {
                n.parse::<u64>().is_ok()
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

    use crate::core::scalar::{Scalar, UInt64Str};
    use crate::{test_scalar_invalid, test_scalar_valid};

    test_scalar_valid! {
        UInt64Str,
        ConstValue::String("100".to_string()),
        ConstValue::String("48464654165168".to_string())
    }

    test_scalar_invalid! {
        UInt64Str,
        ConstValue::Null,
        ConstValue::String("test".to_string()),
        ConstValue::Number(
            Number::from(-15)
        ),
        ConstValue::Number(
            Number::from_f64(1.25).unwrap()
        )
    }
}
