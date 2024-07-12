use indexmap::IndexMap;

use crate::core::json::JsonLike;

pub trait JsonObjectLike {
    type Value<'a>
    where
        Self: 'a;
    fn get_key<'a>(&'a self, key: &str) -> Option<&Self::Value<'a>>;
}

// SerdeValue
impl JsonObjectLike for serde_json::Map<String, serde_json::Value> {
    type Value<'a> = serde_json::Value;
    fn get_key<'a>(&'a self, key: &str) -> Option<&Self::Value<'a>> {
        self.get(key)
    }
}

// ConstValue
impl<V: JsonLike + Clone> JsonObjectLike for IndexMap<async_graphql_value::Name, V> {
    type Value<'a> = V where V: 'a;
    fn get_key<'a>(&'a self, key: &str) -> Option<&Self::Value<'a>> {
        self.get(&async_graphql_value::Name::new(key))
    }
}

