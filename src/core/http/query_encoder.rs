use std::borrow::Cow;

use crate::core::ir::{EvalContext, ResolverContextLike};

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

pub trait Encoder {
    fn encode<T: AsRef<str>, P: AsRef<str>>(
        &self,
        key: T,
        path: &[P],
        encoding_strategy: &EncodingStrategy,
    ) -> Option<String>;
}

impl EncodingStrategy {
    pub fn encode(&self, key: &str, value: Cow<'_, async_graphql::Value>) -> Option<String> {
        match self {
            Self::CommaSeparated => match &*value {
                async_graphql::Value::List(list) => {
                    if list.is_empty() {
                        None
                    } else {
                        Some(format!(
                            "{}={}",
                            key,
                            list.iter()
                                .filter_map(|val| convert_value(Cow::Borrowed(val)))
                                .collect::<Vec<String>>()
                                .join(",")
                        ))
                    }
                }
                _ => encode_value(key, value),
            },
            Self::RepeatedKey => match &*value {
                async_graphql::Value::List(list) => {
                    if list.is_empty() {
                        None
                    } else {
                        let encoded_values: Vec<String> = list
                            .iter()
                            .filter_map(|val| self.encode(key, Cow::Borrowed(val)))
                            .collect();
                        let result = encoded_values.join("&");
                        Some(result)
                    }
                }
                _ => encode_value(key, value),
            },
        }
    }
}

impl<'a, Ctx: ResolverContextLike> Encoder for EvalContext<'a, Ctx> {
    fn encode<T: AsRef<str>, P: AsRef<str>>(
        &self,
        key: T,
        path: &[P],
        encoding_strategy: &EncodingStrategy,
    ) -> Option<String> {
        let ctx = self;

        if path.len() < 2 {
            return None;
        }

        path.split_first()
            .and_then(move |(head, tail)| match head.as_ref() {
                "args" => encoding_strategy.encode(key.as_ref(), ctx.path_arg(tail)?),
                "value" => encoding_strategy.encode(key.as_ref(), ctx.path_value(tail)?),
                "vars" => ctx
                    .var(tail[0].as_ref())
                    .map(|v| format!("{}={}", key.as_ref(), v)),
                "env" => ctx
                    .env_var(tail[0].as_ref())
                    .map(|v| format!("{}={}", key.as_ref(), v)),
                _ => None,
            })
    }
}

pub fn encode_value(key: &str, value: Cow<'_, async_graphql::Value>) -> Option<String> {
    match value {
        Cow::Owned(async_graphql::Value::String(s)) => Some(format!("{}={}", key, s)),
        Cow::Borrowed(async_graphql::Value::String(s)) => Some(format!("{}={}", key, s)),

        Cow::Owned(async_graphql::Value::Number(n)) => Some(format!("{}={}", key, n)),
        Cow::Borrowed(async_graphql::Value::Number(n)) => Some(format!("{}={}", key, n)),

        Cow::Owned(async_graphql::Value::Boolean(b)) => Some(format!("{}={}", key, b)),
        Cow::Borrowed(async_graphql::Value::Boolean(b)) => Some(format!("{}={}", key, b)),

        _ => None,
    }
}

pub fn convert_value(value: Cow<'_, async_graphql::Value>) -> Option<String> {
    match value {
        Cow::Owned(async_graphql::Value::String(s)) => Some(s),
        Cow::Borrowed(async_graphql::Value::String(s)) => Some(s.to_string()),

        Cow::Owned(async_graphql::Value::Number(n)) => Some(n.to_string()),
        Cow::Borrowed(async_graphql::Value::Number(n)) => Some(n.to_string()),

        Cow::Owned(async_graphql::Value::Boolean(b)) => Some(b.to_string()),
        Cow::Borrowed(async_graphql::Value::Boolean(b)) => Some(b.to_string()),

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
