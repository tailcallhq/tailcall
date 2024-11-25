use crate::core::path::ValueString;

/// Defines different strategies for encoding query parameters.
#[derive(Default, Debug, Clone)]
pub enum QueryEncoder {
    /// Encodes the query list as key=value1,value2,value3,...
    CommaSeparated,
    /// Encodes the query list by repeating the key for each value:
    /// key=value1&key=value2&key=value3&...
    #[default]
    RepeatedKey,
}

impl QueryEncoder {
    pub fn encode(&self, key: &str, raw_value: Option<ValueString>) -> String {
        if let Some(value) = raw_value {
            match &value {
                ValueString::Value(val) => self.encode_const_value(key, val),
                ValueString::String(val) => format!("{}={}", key, val),
            }
        } else {
            key.to_owned()
        }
    }
    fn encode_const_value(&self, key: &str, value: &async_graphql::Value) -> String {
        match self {
            QueryEncoder::CommaSeparated => match value {
                async_graphql::Value::List(list) if !list.is_empty() => {
                    let encoded_values: Vec<String> =
                        list.iter().filter_map(convert_value).collect();

                    if encoded_values.is_empty() {
                        key.to_string()
                    } else {
                        format!("{}={}", key, encoded_values.join(","))
                    }
                }
                _ => convert_value(value)
                    .map(|val| format!("{}={}", key, val))
                    .unwrap_or(key.to_string()),
            },
            QueryEncoder::RepeatedKey => match value {
                async_graphql::Value::List(list) if !list.is_empty() => {
                    let encoded_values: Vec<String> = list
                        .iter()
                        .map(|val| self.encode_const_value(key, val))
                        .collect();
                    if encoded_values.is_empty() {
                        key.to_string()
                    } else {
                        encoded_values.join("&")
                    }
                }
                _ => convert_value(value)
                    .map(|val| format!("{}={}", key, val))
                    .unwrap_or(key.to_string()),
            },
        }
    }
}

pub fn convert_value(value: &async_graphql::Value) -> Option<String> {
    match value {
        async_graphql::Value::String(s) => Some(s.to_string()),
        async_graphql::Value::Number(n) => Some(n.to_string()),
        async_graphql::Value::Boolean(b) => Some(b.to_string()),
        async_graphql::Value::Enum(e) => Some(e.to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use async_graphql::Value;

    use super::*;

    #[test]
    fn test_encode_comma_separated_arg() {
        let encoder = QueryEncoder::CommaSeparated;
        let values = Value::List(vec![
            Value::Number(12.into()),
            Value::Number(42.into()),
            Value::Number(13.into()),
        ]);
        let arg_raw_value = Some(ValueString::Value(Cow::Borrowed(&values)));

        let actual = encoder.encode("key", arg_raw_value);
        let expected = "key=12,42,13".to_string();

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_encode_repeated_key_value_arg() {
        let encoder = QueryEncoder::RepeatedKey;
        let values = Value::List(vec![
            Value::Number(12.into()),
            Value::Number(42.into()),
            Value::Number(13.into()),
        ]);
        let arg_raw_value = Some(ValueString::Value(Cow::Borrowed(&values)));

        let actual = encoder.encode("key", arg_raw_value);
        let expected = "key=12&key=42&key=13".to_string();

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_encode_env_var() {
        let encoder = QueryEncoder::default();
        let raw_value = Some(ValueString::String("env_value".into()));

        let actual = encoder.encode("key", raw_value);
        let expected = "key=env_value".to_string();

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_encode_var() {
        let encoder = QueryEncoder::default();
        let raw_value = Some(ValueString::String("var_value".into()));

        let actual = encoder.encode("key", raw_value);
        let expected = "key=var_value".to_string();

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_encode_none() {
        let encoder = QueryEncoder::default();
        let raw_value: Option<ValueString> = None;

        let actual = encoder.encode("key", raw_value);
        let expected = "key".to_owned();

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_encode_comma_separated_strategy() {
        let key = "ids";
        let values = Value::List(vec![
            Value::String("1".to_string()),
            Value::String("2".to_string()),
            Value::String("3".to_string()),
        ]);
        let strategy = QueryEncoder::CommaSeparated;

        let actual = strategy.encode_const_value(key, &values);
        let expected = "ids=1,2,3".to_string();

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
        let strategy = QueryEncoder::RepeatedKey;

        let actual = strategy.encode_const_value(key, &values);
        let expected = "ids=1&ids=2&ids=3".to_string();

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
        let strategy = QueryEncoder::CommaSeparated;

        let actual = strategy.encode_const_value(key, &values);
        let expected = "values=string,42,true".to_string();

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
        let strategy = QueryEncoder::RepeatedKey;

        let actual = strategy.encode_const_value(key, &values);
        let expected = "values=string&values=42&values=true".to_string();

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_encode_empty_list_comma_separated() {
        let key = "empty";
        let values = Value::List(vec![]);
        let strategy = QueryEncoder::CommaSeparated;

        let actual = strategy.encode_const_value(key, &values);
        let expected = "empty".to_string();

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_encode_empty_list_repeated_key() {
        let key = "empty";
        let values = Value::List(vec![]);
        let strategy = QueryEncoder::RepeatedKey;

        let actual = strategy.encode_const_value(key, &values);
        let expected = "empty".to_string();

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_encode_single_value_comma_separated() {
        let key = "single";
        let values = Value::List(vec![Value::String("value".to_string())]);
        let strategy = QueryEncoder::CommaSeparated;

        let actual = strategy.encode_const_value(key, &values);
        let expected = "single=value".to_string();

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_encode_single_value_repeated_key() {
        let key = "single";
        let values = Value::List(vec![Value::String("value".to_string())]);
        let strategy = QueryEncoder::RepeatedKey;

        let actual = strategy.encode_const_value(key, &values);
        let expected = "single=value".to_string();

        assert_eq!(actual, expected);
    }
}
