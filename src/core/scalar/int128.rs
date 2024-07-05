use async_graphql_value::ConstValue;
use schemars::schema::Schema;
use schemars::{schema_for, JsonSchema};

/// Represents signed integer type 128 bit size as string
#[derive(JsonSchema, Default)]
pub struct Int128(pub i128);

impl super::Scalar for Int128 {
    fn validate(&self) -> fn(&ConstValue) -> bool {
        |value| {
            if let ConstValue::String(n) = value {
                n.parse::<i128>().is_ok()
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

    use super::Int128;
    use crate::core::scalar::Scalar;
    use crate::{test_scalar_invalid, test_scalar_valid};

    test_scalar_valid! {
        Int128,
        ConstValue::String("100".to_string()),
        ConstValue::String("-15".to_string()),
        ConstValue::String(i128::MAX.to_string())
    }

    test_scalar_invalid! {
        Int128,
        ConstValue::Null,
        ConstValue::Number(Number::from(15)),
        ConstValue::Number(
            Number::from_f64(1.25).unwrap()
        ),
        ConstValue::String(format!("{}0", i128::MAX))
    }
}
