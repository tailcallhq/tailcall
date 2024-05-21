use async_graphql_value::ConstValue;
use schemars::schema::Schema;
use schemars::{schema_for, JsonSchema};

/// Represents unsigned integer type 64 bit size
#[derive(JsonSchema, Default)]
pub struct UnsignedInt(pub u64);

impl super::Scalar for UnsignedInt {
    fn validate(&self) -> fn(&ConstValue) -> bool {
        |value| {
            if let ConstValue::Number(n) = value {
                n.as_u64().is_some()
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

    use crate::core::scalar::{Scalar, UnsignedInt};

    #[test]
    fn test_unsigned_int_valid() {
        assert!(UnsignedInt::default().validate()(&ConstValue::Number(
            Number::from(100u64)
        )));
    }

    #[test]
    fn test_unsigned_int_invalid_null() {
        assert!(!UnsignedInt::default().validate()(&ConstValue::Null));
    }

    #[test]
    fn test_unsigned_int_invalid_float() {
        assert!(!UnsignedInt::default().validate()(&ConstValue::Number(
            Number::from_f64(1.25).unwrap()
        )));
    }

    #[test]
    fn test_unsigned_int_invalid_signed() {
        assert!(!UnsignedInt::default().validate()(&ConstValue::Number(
            Number::from(-15)
        )));
    }
}
