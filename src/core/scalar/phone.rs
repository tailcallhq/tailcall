use async_graphql_value::ConstValue;
use schemars::schema::Schema;
use schemars::{schema_for, JsonSchema};

use crate::core::json::JsonLike;

/// A field whose value conforms to the standard E.164 format as specified in E.164 specification (https://en.wikipedia.org/wiki/E.164).
#[derive(JsonSchema, Default)]
pub struct PhoneNumber {
    #[allow(dead_code)]
    #[serde(rename = "PhoneNumber")]
    pub phone_no: String,
}
impl super::Scalar for PhoneNumber {
    /// Function used to validate the phone number
    fn validate(&self) -> fn(&ConstValue) -> bool {
        |value| {
            if let Ok(phone_str) = value.clone().as_str_ok() {
                return phonenumber::parse(None, phone_str).is_ok();
            }
            false
        }
    }
    fn schema(&self) -> Schema {
        Schema::Object(schema_for!(Self).schema)
    }
}

#[cfg(test)]
mod test {
    use async_graphql_value::ConstValue;

    use super::*;
    use crate::core::scalar::Scalar;

    #[test]
    fn test_phone_number() {
        let phone = PhoneNumber::default();
        let validate = phone.validate()(&ConstValue::String("+911234567890".to_string()));
        assert!(validate);
    }

    #[test]
    fn test_invalid_phone_number() {
        let phone = PhoneNumber::default();
        let validate = phone.validate()(&ConstValue::String("1234567890".to_string()));
        assert!(!validate);
    }

    #[test]
    fn test_invalid_value() {
        let phone = PhoneNumber::default();
        let validate = phone.validate()(&ConstValue::Null);
        assert!(!validate);
    }
}
