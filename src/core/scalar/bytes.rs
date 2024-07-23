use schemars::JsonSchema;
use tailcall_macros::ScalarDefinition;

use crate::core::json::JsonLikeOwned;

/// Represents list of bytes
#[derive(JsonSchema, Default, ScalarDefinition)]
pub struct Bytes(pub String);

impl super::Scalar for Bytes {
    fn validate<Value: JsonLikeOwned>(&self) -> fn(&Value) -> bool {
        |value| value.as_str().is_some()
    }
}

#[cfg(test)]
mod test {
    use async_graphql_value::ConstValue;
    use serde_json::Number;

    use crate::core::scalar::{Bytes, Scalar};

    #[test]
    fn test_bytes_valid() {
        assert!(Bytes::default().validate()(&ConstValue::String(
            "\0\0".to_string()
        )));
    }

    #[test]
    fn test_bytes_invalid_null() {
        assert!(!Bytes::default().validate()(&ConstValue::Null));
    }

    #[test]
    fn test_bytes_invalid_float() {
        assert!(!Bytes::default().validate()(&ConstValue::Number(
            Number::from_f64(1.25).unwrap()
        )));
    }
}
