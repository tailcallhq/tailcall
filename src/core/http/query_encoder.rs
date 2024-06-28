use async_graphql_value::ConstValue;

use crate::core::ir::{EvalContext, ResolverContextLike};

#[derive(Default)]
pub enum QueryEncoder {
    /// it encodes the query value in the form of
    /// key=value1&key=value2&key=value3
    List,
    /// it encodes the query in the form of key=value
    #[default]
    Single,
}

pub trait Encoder {
    fn encode<T: AsRef<str>>(&self, key: T, value: T) -> String;
}

impl<'a, Ctx: ResolverContextLike> Encoder for EvalContext<'a, Ctx> {
    fn encode<T: AsRef<str>>(&self, key: T, value: T) -> String {
        if let Some(arg_type) = self.path_arg(&[key.as_ref()]) {
            match *arg_type {
                ConstValue::List(_) => QueryEncoder::List.encode(key, value),
                _ => QueryEncoder::Single.encode(key, value),
            }
        } else {
            QueryEncoder::Single.encode(key, value)
        }
    }
}

impl QueryEncoder {
    pub fn encode<K, V>(&self, key: K, values: V) -> String
    where
        K: AsRef<str>,
        V: AsRef<str>,
    {
        match self {
            QueryEncoder::List => values
                .as_ref()
                .trim_start_matches('[')
                .trim_end_matches(']')
                .split(',')
                .map(|v| format!("{}={}", key.as_ref(), v.trim()))
                .collect::<Vec<_>>()
                .join("&"),
            QueryEncoder::Single => format!("{}={}", key.as_ref(), values.as_ref()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_repeated() {
        let encoder = QueryEncoder::List;
        let key = "id";
        let values = "[1,2,3]";
        let encoded = encoder.encode(key, values);
        assert_eq!(encoded, "id=1&id=2&id=3");
    }

    #[test]
    fn test_encode_simple() {
        let encoder = QueryEncoder::Single;
        let key = "q";
        let values = "value";
        let encoded = encoder.encode(key, values);
        assert_eq!(encoded, "q=value");
    }

    #[test]
    fn test_encode_repeated_with_spaces() {
        let encoder = QueryEncoder::List;
        let key = "id";
        let values = "[ 1 , 2 , 3 ]";
        let encoded = encoder.encode(key, values);
        assert_eq!(encoded, "id=1&id=2&id=3");
    }
}
