use crate::core::json::{gather_path_matches, group_by_key, JsonLike, JsonSchema};
use crate::core::valid::Valid;
use async_graphql::dynamic::{FieldFuture, FieldValue};
use async_graphql_value::{ConstValue, Name};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub struct Value(async_graphql::Value);

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = async_graphql::Value::deserialize(deserializer)?;
        Ok(Value(value))
    }
}

impl Default for Value {
    fn default() -> Self {
        Value(async_graphql::Value::Null)
    }
}

impl Default for &Value {
    fn default() -> Self {
        &Value(async_graphql::Value::Null)
    }
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl From<async_graphql::Value> for Value {
    fn from(value: async_graphql::Value) -> Self {
        Value(value)
    }
}

impl Value {
    pub fn validate_with(&self, schema: &JsonSchema) -> Valid<(), &'static str> {
        todo!()
    }

    pub fn from_value_borrow(value: &'_ async_graphql::Value) -> &'_ Self {
        todo!()
    }

    pub fn from_borrowed_list<'a>(list: &'a Vec<async_graphql::Value>) -> &'a Vec<Value> {
        todo!()
    }

    pub fn into_field_future<'a>(input: Option<Value>) -> FieldFuture<'a> {
        FieldFuture::from_value(input.map(|v| v.0))
    }

    pub fn as_fieldValue<'a>(self) -> Option<FieldValue<'a>> {
        match self.0 {
            ConstValue::Null => FieldValue::NONE,
            ConstValue::List(a) => Some(FieldValue::list(a)),
            a => Some(FieldValue::from(a)),
        }
    }

    pub fn list(self) -> Option<Vec<Value>> {
        match self.0 {
            async_graphql::Value::List(list) => Some(list.into_iter().map(Value).collect()),
            _ => None,
        }
    }

    pub fn as_list(list: Vec<Value>) -> Value {
        Value(async_graphql::Value::List(
            list.into_iter().map(|v| v.0).collect(),
        ))
    }

    pub fn as_object(obj: IndexMap<Name, &Value>) -> Self {
        Value(async_graphql::Value::Object(
            obj.into_iter()
                .map(|(k, v)| (k.clone(), v.to_owned().0))
                .collect(),
        ))
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        match &self.0 {
            ConstValue::List(map) => {
                let key = key.parse::<usize>().ok()?;
                map.get(key).map(Value::from_value_borrow)
            }
            ConstValue::Object(map) => map.get(key).map(Value::from_value_borrow),
            _ => None,
        }
    }
    pub fn convert_value(value: Cow<'_, Value>) -> Option<Cow<'_, str>> {
        match &value.0 {
            async_graphql::Value::Number(n) => Some(Cow::Owned(n.to_string())),
            async_graphql::Value::Boolean(b) => Some(Cow::Owned(b.to_string())),
            async_graphql::Value::Object(map) => Some(json!(map).to_string().into()),
            async_graphql::Value::List(list) => Some(json!(list).to_string().into()),
            async_graphql::Value::String(s) => Some(Cow::Owned(s.to_owned())),
            _ => None,
        }
    }
}

impl JsonLike for Value {
    type Output = Value;

    fn as_array_ok(&self) -> Result<&Vec<Self::Output>, &str> {
        match &self.0 {
            ConstValue::List(seq) => Ok(Value::from_borrowed_list(seq)),
            _ => Err("array"),
        }
    }

    fn as_str_ok(&self) -> Result<&str, &str> {
        match &self.0 {
            ConstValue::String(s) => Ok(s),
            _ => Err("str"),
        }
    }

    fn as_i64_ok(&self) -> Result<i64, &str> {
        match &self.0 {
            ConstValue::Number(n) => n.as_i64().ok_or("expected i64"),
            _ => Err("i64"),
        }
    }

    fn as_u64_ok(&self) -> Result<u64, &str> {
        match &self.0 {
            ConstValue::Number(n) => n.as_u64().ok_or("expected u64"),
            _ => Err("u64"),
        }
    }

    fn as_f64_ok(&self) -> Result<f64, &str> {
        match &self.0 {
            ConstValue::Number(n) => n.as_f64().ok_or("expected f64"),
            _ => Err("f64"),
        }
    }

    fn as_bool_ok(&self) -> Result<bool, &str> {
        match &self.0 {
            ConstValue::Boolean(b) => Ok(*b),
            _ => Err("bool"),
        }
    }

    fn as_null_ok(&self) -> Result<(), &str> {
        match &self.0 {
            ConstValue::Null => Ok(()),
            _ => Err("null"),
        }
    }

    fn as_option_ok(&self) -> Result<Option<&Self::Output>, &str> {
        match &self.0 {
            ConstValue::Null => Ok(None),
            _ => Ok(Some(self)),
        }
    }

    fn get_path<T: AsRef<str>>(&self, path: &[T]) -> Option<&Self::Output> {
        let mut val = self;
        for token in path {
            val = val.get(token.as_ref())?;
        }
        Some(val)
    }

    fn new(value: &Self::Output) -> &Self {
        value
    }

    fn get_key(&self, path: &str) -> Option<&Self::Output> {
        match &self.0 {
            ConstValue::Object(map) => map
                .get(&async_graphql::Name::new(path))
                .map(Value::from_value_borrow),
            _ => None,
        }
    }
    fn as_string_ok(&self) -> Result<&String, &str> {
        match &self.0 {
            ConstValue::String(s) => Ok(s),
            _ => Err("expected string"),
        }
    }

    fn group_by<'a>(&'a self, path: &'a [String]) -> HashMap<String, Vec<&'a Self::Output>> {
        let src = gather_path_matches(self, path, vec![]);
        group_by_key(src)
    }
}

// impl From<Value> for FieldFuture<'_> {
//     fn from(value: Value) -> Self {
//         FieldFuture::new(value.0)
//     }
// }
