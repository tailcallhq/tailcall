use indexmap::IndexMap;
use serde_json_borrow::{ObjectAsVec, Value};

use crate::core::json::JsonLike;

pub trait JsonObjectLike {
    type Value<'a>: JsonLike<'a>
    where
        Self: 'a;
    fn get<'a>(&'a self, key: &'a str) -> Option<&Self::Value<'a>>;
}

impl JsonObjectLike for serde_json::Map<String, serde_json::Value> {
    type Value<'a> = serde_json::Value;

    fn get<'a>(&'a self, key: &'a str) -> Option<&Self::Value<'a>> {
        self.get(key)
    }
}

impl<V: for<'json> JsonLike<'json> + Clone> JsonObjectLike
    for IndexMap<async_graphql_value::Name, V>
{
    type Value<'a> = V where V: 'a;

    fn get<'a>(&'a self, key: &'a str) -> Option<&Self::Value<'a>> {
        self.get(&async_graphql_value::Name::new(key))
    }
}

impl<'x> JsonObjectLike for ObjectAsVec<'x> {
    type Value<'a> = Value<'a> where 'x: 'a;

    fn get<'a>(&'a self, key: &'a str) -> Option<&'a Self::Value<'a>> {
        self.get(key)
    }
}
