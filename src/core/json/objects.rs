use indexmap::IndexMap;
use serde_json_borrow::{ObjectAsVec, Value};

use crate::core::json::JsonLike;

pub trait Object {
    type Value: JsonLike;
    // TODO: Should return a reference to the value
    fn get_val(&self, key: &str) -> Option<Self::Value>;
    fn insert_val(&mut self, key: &str, value: Self::Value);
}

impl Object for serde_json::Map<String, serde_json::Value> {
    type Value = serde_json::Value;

    fn get_val(&self, key: &str) -> Option<Self::Value> {
        self.get(key).cloned()
    }

    fn insert_val(&mut self, key: &str, value: Self::Value) {
        self.insert(key.to_string(), value);
    }
}

impl<V: JsonLike + Clone> Object for IndexMap<async_graphql_value::Name, V> {
    type Value = V;

    fn get_val(&self, key: &str) -> Option<Self::Value> {
        self.get(&async_graphql_value::Name::new(key)).cloned()
    }

    fn insert_val(&mut self, key: &str, value: Self::Value) {
        self.insert(async_graphql_value::Name::new(key), value);
    }
}

impl<'a> Object for ObjectAsVec<'a> {
    type Value = Value<'a>;

    fn get_val(&self, key: &str) -> Option<Self::Value> {
        self.clone()
            .into_vec()
            .into_iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v)
    }

    fn insert_val(&mut self, _key: &str, _value: Self::Value) {
        todo!()
        // self.insert(key, value);
    }
}
