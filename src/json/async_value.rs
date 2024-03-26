use async_graphql_value::ConstValue;

use super::*;

impl JsonLike for async_graphql::Value {
    type Output = async_graphql::Value;

    fn as_array_ok(&self) -> Option<&Vec<Self::Output>> {
        match self {
            ConstValue::List(seq) => Some(seq),
            _ => None,
        }
    }

    fn as_str_ok(&self) -> Option<&str> {
        match self {
            ConstValue::String(s) => Some(s),
            _ => None,
        }
    }

    fn as_i64_ok(&self) -> Option<i64> {
        match self {
            ConstValue::Number(n) => n.as_i64(),
            _ => None,
        }
    }

    fn as_u64_ok(&self) -> Option<u64> {
        match self {
            ConstValue::Number(n) => n.as_u64(),
            _ => None,
        }
    }

    fn as_f64_ok(&self) -> Option<f64> {
        match self {
            ConstValue::Number(n) => n.as_f64(),
            _ => None,
        }
    }

    fn as_bool_ok(&self) -> Option<bool> {
        match self {
            ConstValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    fn as_null_ok(&self) -> Option<()> {
        match self {
            ConstValue::Null => Some(()),
            _ => None,
        }
    }

    fn get_path<T: AsRef<str>>(&self, path: &[T]) -> Option<&Self::Output> {
        let mut val = self;
        for token in path {
            val = match val {
                ConstValue::List(seq) => {
                    let index = token.as_ref().parse::<usize>().ok()?;
                    seq.get(index)?
                }
                ConstValue::Object(map) => map.get(token.as_ref())?,
                _ => return None,
            };
        }
        Some(val)
    }

    fn as_string_ok(&self) -> Option<&String> {
        match self {
            ConstValue::String(s) => Some(s),
            _ => None,
        }
    }
}
