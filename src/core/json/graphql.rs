use std::collections::HashMap;

use async_graphql::Name;
use async_graphql_value::ConstValue;
use indexmap::IndexMap;

use super::*;

impl<'ctx, Value: JsonLike + Clone> JsonObjectLike<'ctx> for IndexMap<Name, Value> {
    type Value<'json> = Value
    where
        Self: 'json,
        'json: 'ctx;

    fn new() -> Self {
        IndexMap::new()
    }

    fn get_key<'a: 'ctx>(&'a self, key: &str) -> Option<&Self::Value<'a>> {
        self.get(&Name::new(key))
    }

    fn insert_key<'a: 'ctx>(&mut self, key: &'a str, value: Self::Value<'a>)
    where
        Self: 'a,
    {
        self.insert(Name::new(key), value);
    }
}

impl JsonLike for ConstValue {
    type JsonObject<'a> = IndexMap<Name, ConstValue>;
    type Output<'a>  = ConstValue where Self: 'a;

    fn null() -> Self {
        Default::default()
    }

    fn as_array(&self) -> Option<&Vec<Self>> {
        match self {
            ConstValue::List(seq) => Some(seq),
            _ => None,
        }
    }

    fn as_object(&self) -> Option<&Self::JsonObject<'_>> {
        match self {
            ConstValue::Object(map) => Some(map),
            _ => None,
        }
    }

    fn as_str(&self) -> Option<&str> {
        match self {
            ConstValue::String(s) => Some(s),
            _ => None,
        }
    }

    fn as_i64(&self) -> Option<i64> {
        match self {
            ConstValue::Number(n) => n.as_i64(),
            _ => None,
        }
    }

    fn as_u64(&self) -> Option<u64> {
        match self {
            ConstValue::Number(n) => n.as_u64(),
            _ => None,
        }
    }

    fn as_f64(&self) -> Option<f64> {
        match self {
            ConstValue::Number(n) => n.as_f64(),
            _ => None,
        }
    }

    fn as_bool(&self) -> Option<bool> {
        match self {
            ConstValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    fn is_null(&self) -> bool {
        matches!(self, ConstValue::Null)
    }

    fn get_path<'a, T: AsRef<str>>(&'a self, path: &'a [T]) -> Option<&Self::Output<'a>> {
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

    fn get_key<'a>(&'a self, path: &'a str) -> Option<&Self::Output<'a>> {
        match self {
            ConstValue::Object(map) => map.get(&async_graphql::Name::new(path)),
            _ => None,
        }
    }

    fn group_by<'a>(&'a self, path: &'a [String]) -> HashMap<String, Vec<&'a Self::Output<'a>>> {
        let src = gather_path_matches(self, path, vec![]);
        group_by_key(src)
    }
}
