use std::fmt::{Display, Formatter};

use async_graphql_value::ConstValue;
use chrono::DateTime;
use schemars::JsonSchema;

use crate::json::JsonLike;

#[derive(JsonSchema, Default)]
pub struct Date {
    #[serde(rename = "Date")]
    /// A date string, such as 2007-12-03, is compliant with the full-date format outlined in section 5.6 of the RFC 3339 (https://datatracker.ietf.org/doc/html/rfc3339) profile of the ISO 8601 standard for the representation of dates and times using the Gregorian calendar.
    pub date: String,
}

impl Display for Date {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Date")
    }
}

impl super::Scalar for Date {
    /// Function used to validate the date
    fn validate(&self) -> fn(&ConstValue) -> bool {
        |value| {
            if let Ok(date_str) = value.clone().as_str_ok() {
                return DateTime::parse_from_rfc3339(date_str).is_ok();
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
    fn test_date() {
        let date = Date::default();
        let validate = date.validate()(&ConstValue::String("2020-01-01T12:00:00Z".to_string()));
        assert!(validate);
    }

    #[test]
    fn test_invalid_date() {
        let date = Date::default();
        let validate = date.validate()(&ConstValue::String("2023-03-08T12:45:26".to_string()));
        assert!(!validate);
    }

    #[test]
    fn test_invalid_value() {
        let date = Date::default();
        let validate = date.validate()(&ConstValue::Null);
        assert!(!validate);
    }
}
