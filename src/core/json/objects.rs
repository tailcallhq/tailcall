use indexmap::IndexMap;
use serde_json_borrow::{ObjectAsVec, Value};
use crate::core::json::JsonLike;

pub trait Object<'ctx> {
    type Value: JsonLike;
    fn get_val(&'ctx self, key: &str) -> Option<&'ctx Self::Value>;
    fn insert_val(&'ctx mut self, key: &'ctx str, value: Self::Value);
}

impl<'ctx> Object<'ctx> for serde_json::Map<String, serde_json::Value> {
    type Value = serde_json::Value;

    fn get_val(&'ctx self, key: &str) -> Option<&'ctx Self::Value> {
        self.get(key)
    }

    fn insert_val(&'ctx mut self, key: &'ctx str, value: Self::Value) {
        self.insert(key.to_string(), value);
    }
}

impl<'ctx, V: JsonLike> Object<'ctx> for IndexMap<async_graphql_value::Name, V> {
    type Value = V;

    fn get_val(&'ctx self, key: &str) -> Option<&'ctx Self::Value> {
        self.get(&async_graphql_value::Name::new(key))
    }

    fn insert_val(&'ctx mut self, key: &'ctx str, value: Self::Value) {
        self.insert(async_graphql_value::Name::new(key), value);
    }
}

impl<'ctx> Object<'ctx> for ObjectAsVec<'ctx> {
    type Value = Value<'ctx>;

    fn get_val(&'ctx self, key: &str) -> Option<&'ctx Self::Value> {
        self.get(key)
    }

    fn insert_val(&'ctx mut self, key: &'ctx str, value: Self::Value) {
        self.insert(key, value);
    }
}