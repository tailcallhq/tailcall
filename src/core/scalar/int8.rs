use async_graphql_value::ConstValue;
use schemars::schema::Schema;
use schemars::{schema_for, JsonSchema};

/// Represents signed integer type 8bit size
#[derive(JsonSchema, Default)]
pub struct Int8(pub i8);

impl super::Scalar for Int8 {
    fn validate(&self) -> fn(&ConstValue) -> bool {
        |value| {
            if let ConstValue::Number(n) = value {
                if let Some(n) = n.as_i64() {
                    return i8::try_from(n).is_ok();
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
