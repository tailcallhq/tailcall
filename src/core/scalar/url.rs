use async_graphql_value::ConstValue;
use schemars::schema::Schema;
use schemars::{schema_for, JsonSchema};

use crate::core::json::JsonLike;

/// A field whose value conforms to the standard URL format as specified in RFC3986 (https://www.ietf.org/rfc/rfc3986.txt), and it uses real JavaScript URL objects.
#[derive(JsonSchema, Default)]
pub struct Url {
    #[allow(dead_code)]
    #[serde(rename = "Url")]
    pub url: String,
}

impl super::Scalar for Url {
    /// Function used to validate the date
    fn validate(&self) -> fn(&ConstValue) -> bool {
        |value| {
            if let Ok(date_str) = value.clone().as_str_ok() {
                return url::Url::parse(date_str).is_ok();
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
    fn test_url() {
        let date = Url::default();
        let validate = date.validate()(&ConstValue::String("https://ssdd.dev".to_string()));
        assert!(validate);
    }

    #[test]
    fn test_invalid_url() {
        let date = Url::default();
        let validate = date.validate()(&ConstValue::String("localhost".to_string()));
        assert!(!validate);
    }

    #[test]
    fn test_invalid_value() {
        let date = Url::default();
        let validate = date.validate()(&ConstValue::Null);
        assert!(!validate);
    }
}
