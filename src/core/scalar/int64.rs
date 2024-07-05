use async_graphql_value::ConstValue;
use schemars::schema::Schema;
use schemars::{schema_for, JsonSchema};

/// Represents signed integer type 64bit size as string
#[derive(JsonSchema, Default)]
pub struct Int64(pub i64);

impl super::Scalar for Int64 {
    fn validate(&self) -> fn(&ConstValue) -> bool {
        |value| {
            if let ConstValue::String(n) = value {
                n.parse::<i64>().is_ok()
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

    use super::Int64;
    use crate::core::scalar::Scalar;
    use crate::{test_scalar_invalid, test_scalar_valid};

    test_scalar_valid! {
        Int64,
        ConstValue::String("125".to_string()),
        ConstValue::String("-15".to_string()),
        ConstValue::String(i64::MAX.to_string())
    }

    test_scalar_invalid! {
        Int64,
        ConstValue::Null,
        ConstValue::Number(Number::from(15)),
        ConstValue::Number(
            Number::from_f64(1.25).unwrap()
        ),
        ConstValue::String(format!("{}1", i64::MAX))
    }
}
