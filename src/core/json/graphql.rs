use std::collections::HashMap;

use async_graphql::Name;
use async_graphql_value::ConstValue;
use indexmap::IndexMap;

use super::*;

impl<'a, Value: JsonLike<'a> + Clone> JsonObjectLike<'a> for IndexMap<Name, Value> {
    type Value = Value;
    fn get_key(&'a self, key: &str) -> Option<&Self::Value> {
        self.get(&Name::new(key))
    }
}

impl<'a> JsonArrayLike<'a> for Vec<ConstValue> {
    type Value = ConstValue;
    fn as_vec(&'a self) -> &'a Vec<Self::Value> {
        self
    }
}

impl<'a> JsonLike<'a> for ConstValue {
    type JsonObject = IndexMap<Name, ConstValue>;
    type JsonArray = Vec<ConstValue>;

    fn as_array_ok(&'a self) -> Result<&Self::JsonArray, &str> {
        match self {
            ConstValue::List(seq) => Ok(seq),
            _ => Err("array"),
        }
    }

    fn as_str_ok(&self) -> Result<&str, &str> {
        match self {
            ConstValue::String(s) => Ok(s),
            _ => Err("str"),
        }
    }

    fn as_i64_ok(&self) -> Result<i64, &str> {
        match self {
            ConstValue::Number(n) => n.as_i64().ok_or("expected i64"),
            _ => Err("i64"),
        }
    }

    fn as_u64_ok(&self) -> Result<u64, &str> {
        match self {
            ConstValue::Number(n) => n.as_u64().ok_or("expected u64"),
            _ => Err("u64"),
        }
    }

    fn as_f64_ok(&self) -> Result<f64, &str> {
        match self {
            ConstValue::Number(n) => n.as_f64().ok_or("expected f64"),
            _ => Err("f64"),
        }
    }

    fn as_bool_ok(&self) -> Result<bool, &str> {
        match self {
            ConstValue::Boolean(b) => Ok(*b),
            _ => Err("bool"),
        }
    }

    fn as_null_ok(&self) -> Result<(), &str> {
        match self {
            ConstValue::Null => Ok(()),
            _ => Err("null"),
        }
    }

    fn as_option_ok(&self) -> Result<Option<&Self>, &str> {
        match self {
            ConstValue::Null => Ok(None),
            _ => Ok(Some(self)),
        }
    }

    fn get_path<T: AsRef<str>>(&self, path: &[T]) -> Option<&Self> {
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

    fn get_key(&self, path: &str) -> Option<&Self> {
        match self {
            ConstValue::Object(map) => map.get(&async_graphql::Name::new(path)),
            _ => None,
        }
    }

    fn group_by(&'a self, path: &'a [String]) -> HashMap<String, Vec<&'a Self>> {
        let src = gather_path_matches(self, path, vec![]);
        group_by_key(src)
    }

    fn null() -> Self {
        Default::default()
    }

    fn as_object_ok(&'a self) -> Result<&Self::JsonObject, &str> {
        match self {
            ConstValue::Object(map) => Ok(map),
            _ => Err("expected object"),
        }
    }
}
