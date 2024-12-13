use std::borrow::Cow;
use std::collections::HashMap;

use async_graphql::Name;
use async_graphql_value::{ConstValue, Value};
use indexmap::IndexMap;

use super::*;

impl<'obj, Value: JsonLike<'obj>> JsonObjectLike<'obj> for IndexMap<Name, Value> {
    type Value = Value;

    fn new() -> Self {
        IndexMap::new()
    }

    fn with_capacity(n: usize) -> Self {
        IndexMap::with_capacity(n)
    }

    fn from_vec(v: Vec<(&'obj str, Self::Value)>) -> Self {
        IndexMap::from_iter(v.into_iter().map(|(k, v)| (Name::new(k), v)))
    }

    fn get_key(&self, key: &str) -> Option<&Self::Value> {
        self.get(key)
    }

    fn insert_key(&mut self, key: &'obj str, value: Self::Value) {
        self.insert(Name::new(key), value);
    }

    fn iter<'slf>(&'slf self) -> impl Iterator<Item = (&'slf str, &'slf Self::Value)>
    where
        Self::Value: 'slf,
        'obj: 'slf,
    {
        self.iter().map(|(k, v)| (k.as_str(), v))
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

impl<'json> JsonLike<'json> for ConstValue {
    type JsonObject = IndexMap<Name, ConstValue>;

    fn from_primitive(x: JsonPrimitive<'json>) -> Self {
        match x {
            JsonPrimitive::Null => ConstValue::Null,
            JsonPrimitive::Bool(x) => ConstValue::Boolean(x),
            JsonPrimitive::Str(s) => ConstValue::String(s.to_string()),
            JsonPrimitive::Number(number) => ConstValue::Number(number),
        }
    }

    fn as_primitive(&self) -> Option<JsonPrimitive> {
        let val = match self {
            ConstValue::Null => JsonPrimitive::Null,
            ConstValue::Boolean(x) => JsonPrimitive::Bool(*x),
            ConstValue::Number(number) => JsonPrimitive::Number(number.clone()),
            ConstValue::String(s) => JsonPrimitive::Str(s.as_ref()),
            ConstValue::Enum(e) => JsonPrimitive::Str(e.as_str()),
            _ => return None,
        };

        Some(val)
    }

    fn as_array(&self) -> Option<&Vec<Self>> {
        match self {
            ConstValue::List(seq) => Some(seq),
            _ => None,
        }
    }

    fn as_array_mut(&mut self) -> Option<&mut Vec<Self>> {
        match self {
            ConstValue::List(seq) => Some(seq),
            _ => None,
        }
    }

    fn into_array(self) -> Option<Vec<Self>> {
        match self {
            ConstValue::List(seq) => Some(seq),
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

    fn group_by(&self, path: &[String]) -> HashMap<String, Vec<&Self>> {
        let src = gather_path_matches(self, path, vec![]);
        group_by_key(src)
    }

    fn null() -> Self {
        Default::default()
    }

    fn as_object(&self) -> Option<&Self::JsonObject> {
        match self {
            ConstValue::Object(map) => Some(map),
            _ => None,
        }
    }

    fn as_object_mut(&mut self) -> Option<&mut Self::JsonObject> {
        match self {
            ConstValue::Object(map) => Some(map),
            _ => None,
        }
    }

    fn into_object(self) -> Option<Self::JsonObject> {
        match self {
            ConstValue::Object(map) => Some(map),
            _ => None,
        }
    }

    fn object(obj: Self::JsonObject) -> Self {
        ConstValue::Object(obj)
    }

    fn array(arr: Vec<Self>) -> Self {
        ConstValue::List(arr)
    }

    fn string(s: Cow<'json, str>) -> Self {
        ConstValue::String(s.to_string())
    }
}

impl<'json> JsonLike<'json> for Value {
    type JsonObject = IndexMap<Name, Value>;

    fn from_primitive(x: JsonPrimitive<'json>) -> Self {
        match x {
            JsonPrimitive::Null => Value::Null,
            JsonPrimitive::Bool(x) => Value::Boolean(x),
            JsonPrimitive::Str(s) => Value::String(s.to_string()),
            JsonPrimitive::Number(number) => Value::Number(number),
        }
    }

    fn as_primitive(&self) -> Option<JsonPrimitive> {
        let val = match self {
            Value::Null => JsonPrimitive::Null,
            Value::Boolean(x) => JsonPrimitive::Bool(*x),
            Value::Number(number) => JsonPrimitive::Number(number.clone()),
            Value::String(s) => JsonPrimitive::Str(s.as_ref()),
            Value::Enum(e) => JsonPrimitive::Str(e.as_str()),
            _ => return None,
        };

        Some(val)
    }

    fn as_array(&self) -> Option<&Vec<Self>> {
        match self {
            Value::List(seq) => Some(seq),
            _ => None,
        }
    }

    fn as_array_mut(&mut self) -> Option<&mut Vec<Self>> {
        match self {
            Value::List(seq) => Some(seq),
            _ => None,
        }
    }

    fn into_array(self) -> Option<Vec<Self>> {
        match self {
            Value::List(seq) => Some(seq),
            _ => None,
        }
    }

    fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Number(n) => n.as_i64(),
            _ => None,
        }
    }

    fn as_u64(&self) -> Option<u64> {
        match self {
            Value::Number(n) => n.as_u64(),
            _ => None,
        }
    }

    fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Number(n) => n.as_f64(),
            _ => None,
        }
    }

    fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    fn get_path<T: AsRef<str>>(&self, path: &[T]) -> Option<&Self> {
        let mut val = self;
        for token in path {
            val = match val {
                Value::List(seq) => {
                    let index = token.as_ref().parse::<usize>().ok()?;
                    seq.get(index)?
                }
                Value::Object(map) => map.get(token.as_ref())?,
                _ => return None,
            };
        }
        Some(val)
    }

    fn get_key(&self, path: &str) -> Option<&Self> {
        match self {
            Value::Object(map) => map.get(&async_graphql::Name::new(path)),
            _ => None,
        }
    }

    fn group_by(&self, path: &[String]) -> HashMap<String, Vec<&Self>> {
        let src = gather_path_matches(self, path, vec![]);
        group_by_key(src)
    }

    fn null() -> Self {
        Default::default()
    }

    fn as_object(&self) -> Option<&Self::JsonObject> {
        match self {
            Value::Object(map) => Some(map),
            _ => None,
        }
    }

    fn as_object_mut(&mut self) -> Option<&mut Self::JsonObject> {
        match self {
            Value::Object(map) => Some(map),
            _ => None,
        }
    }

    fn into_object(self) -> Option<Self::JsonObject> {
        match self {
            Value::Object(map) => Some(map),
            _ => None,
        }
    }

    fn object(obj: Self::JsonObject) -> Self {
        Value::Object(obj)
    }

    fn array(arr: Vec<Self>) -> Self {
        Value::List(arr)
    }

    fn string(s: Cow<'json, str>) -> Self {
        Value::String(s.to_string())
    }
}
