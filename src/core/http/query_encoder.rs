use std::borrow::Cow;

use crate::core::path::RawValue;

/// Defines different strategies for encoding query parameters.
#[derive(Default, Debug, Clone)]
pub enum EncodingStrategy {
    /// Encodes the query list as key=value1,value2,value3,...
    CommaSeparated,
    /// Encodes the query list by repeating the key for each value:
    /// key=value1&key=value2&key=value3&...
    #[default]
    RepeatedKey,
}

#[derive(Default, Debug, Clone)]
pub struct QueryEncoder {
    encoding_strategy: EncodingStrategy,
}

impl QueryEncoder {
    pub fn encode(&self, key: &str, value: &[RawValue]) -> Option<String> {
        if value.is_empty() {
            return None;
        }

        match &value[0] {
            RawValue::Arg(arg) => self.encoding_strategy.encode(key, Cow::Borrowed(arg)),
            RawValue::Value(val) => self.encoding_strategy.encode(key, Cow::Borrowed(val)),
            RawValue::Env(env_var) => Some(format!("{}={}", key, env_var)),
            RawValue::Headers(headers_value) => Some(format!("{}={}", key, headers_value)),
            RawValue::Var(var) => Some(format!("{}={}", key, var)),
            _ => None,
        }
    }
}

impl EncodingStrategy {
    pub fn encode(&self, key: &str, value: Cow<'_, async_graphql::Value>) -> Option<String> {
        match self {
            EncodingStrategy::CommaSeparated => match &*value {
                async_graphql::Value::List(list) if !list.is_empty() => {
                    let encoded_values: Vec<String> = list
                        .iter()
                        .filter_map(|val| convert_value(Cow::Borrowed(val)))
                        .collect();
                    Some(format!("{}={}", key, encoded_values.join(",")))
                }
                _ => convert_value(value).map(|val| format!("{}={}", key, val)),
            },
            EncodingStrategy::RepeatedKey => match &*value {
                async_graphql::Value::List(list) if !list.is_empty() => {
                    let encoded_values: Vec<String> = list
                        .iter()
                        .filter_map(|val| self.encode(key, Cow::Borrowed(val)))
                        .collect();
                    Some(encoded_values.join("&"))
                }
                _ => convert_value(value).map(|val| format!("{}={}", key, val)),
            },
        }
    }
}

pub fn convert_value(value: Cow<'_, async_graphql::Value>) -> Option<String> {
    match &*value {
        async_graphql::Value::String(s) => Some(s.to_string()),
        async_graphql::Value::Number(n) => Some(n.to_string()),
        async_graphql::Value::Boolean(b) => Some(b.to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use async_graphql::Value;

    use super::*;
    #[test]
    fn test_encode_comma_separated_strategy() {
        let key = "ids";
        let values = Value::List(vec![
            Value::String("1".to_string()),
            Value::String("2".to_string()),
            Value::String("3".to_string()),
        ]);
        let strategy = EncodingStrategy::CommaSeparated;

        let actual = strategy.encode(key, Cow::Owned(values));
        let expected = Some("ids=1,2,3".to_string());

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_encode_repeated_key_strategy() {
        let key = "ids";
        let values = Value::List(vec![
            Value::String("1".to_string()),
            Value::String("2".to_string()),
            Value::String("3".to_string()),
        ]);
        let strategy = EncodingStrategy::RepeatedKey;

        let actual = strategy.encode(key, Cow::Owned(values));
        let expected = Some("ids=1&ids=2&ids=3".to_string());

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_encode_mixed_values_comma_separated() {
        let key = "values";
        let values = Value::List(vec![
            Value::String("string".to_string()),
            Value::Number(42.into()),
            Value::Boolean(true),
        ]);
        let strategy = EncodingStrategy::CommaSeparated;

        let actual = strategy.encode(key, Cow::Owned(values));
        let expected = Some("values=string,42,true".to_string());

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_encode_mixed_values_repeated_key() {
        let key = "values";
        let values = Value::List(vec![
            Value::String("string".to_string()),
            Value::Number(42.into()),
            Value::Boolean(true),
        ]);
        let strategy = EncodingStrategy::RepeatedKey;

        let actual = strategy.encode(key, Cow::Owned(values));
        let expected = Some("values=string&values=42&values=true".to_string());

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_encode_empty_list_comma_separated() {
        let key = "empty";
        let values = Value::List(vec![]);
        let strategy = EncodingStrategy::CommaSeparated;

        let actual = strategy.encode(key, Cow::Owned(values));
        let expected: Option<String> = None;

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_encode_empty_list_repeated_key() {
        let key = "empty";
        let values = Value::List(vec![]);
        let strategy = EncodingStrategy::RepeatedKey;

        let actual = strategy.encode(key, Cow::Owned(values));
        let expected: Option<String> = None;

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_encode_single_value_comma_separated() {
        let key = "single";
        let values = Value::List(vec![Value::String("value".to_string())]);
        let strategy = EncodingStrategy::CommaSeparated;

        let actual = strategy.encode(key, Cow::Owned(values));
        let expected = Some("single=value".to_string());

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_encode_single_value_repeated_key() {
        let key = "single";
        let values = Value::List(vec![Value::String("value".to_string())]);
        let strategy = EncodingStrategy::RepeatedKey;

        let actual = strategy.encode(key, Cow::Owned(values));
        let expected = Some("single=value".to_string());

        assert_eq!(actual, expected);
    }
}
