use std::fmt::{Display, Formatter};

use async_graphql_value::ConstValue;
use schemars::JsonSchema;

use crate::json::JsonLike;

#[derive(JsonSchema, Default)]
pub struct Url {
    #[serde(rename = "Url")]
    /// A field whose value conforms to the standard URL format as specified in RFC3986 (https://www.ietf.org/rfc/rfc3986.txt), and it uses real JavaScript URL objects.
    pub url: String,
}

impl Display for Url {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Url")
    }
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
}

#[cfg(test)]
mod test {
    use async_graphql_value::ConstValue;

    use super::*;
    use crate::scalar::Scalar;

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
