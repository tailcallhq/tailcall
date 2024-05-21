use async_graphql_value::ConstValue;
use schemars::schema::Schema;
use schemars::{schema_for, JsonSchema};

/// Represents signed integer type 64 bit size
#[derive(JsonSchema, Default)]
pub struct BigInt(pub i64);

impl super::Scalar for BigInt {
    fn validate(&self) -> fn(&ConstValue) -> bool {
        |value| {
            if let ConstValue::Number(n) = value {
                !n.is_f64()
            } else {
                false
            }
        }
    }

    fn scalar(&self) -> Schema {
        schema_for!(Self).schema.into()
    }
}

#[cfg(test)]
mod test {
    use async_graphql_value::ConstValue;
    use serde_json::Number;

    use crate::core::scalar::{BigInt, Scalar};

    #[test]
    fn test_big_int_valid() {
        assert!(BigInt::default().validate()(&ConstValue::Number(
            Number::from(100u64)
        )));
    }

    #[test]
    fn test_big_int_invalid_null() {
        assert!(!BigInt::default().validate()(&ConstValue::Null));
    }

    #[test]
    fn test_big_int_invalid_float() {
        assert!(!BigInt::default().validate()(&ConstValue::Number(
            Number::from_f64(1.25).unwrap()
        )));
    }
}
