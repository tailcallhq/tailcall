use indexmap::IndexMap;
use serde_json_borrow::{ObjectAsVec, Value as BorrowedValue};
use simd_json::borrowed::Object as SimdObject;
use simd_json::BorrowedValue as SimdBorrowedValue;

use crate::core::json::JsonLike;

pub trait JsonObjectLike {
    type Value<'a>: JsonLike
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

impl<V: JsonLike + Clone> JsonObjectLike for IndexMap<async_graphql_value::Name, V> {
    type Value<'a> = V where V: 'a;

    fn get<'a>(&'a self, key: &'a str) -> Option<&Self::Value<'a>> {
        self.get(&async_graphql_value::Name::new(key))
    }
}

impl<'ctx> JsonObjectLike for ObjectAsVec<'ctx> {
    type Value<'a> = BorrowedValue<'a> where 'ctx: 'a;

    fn get<'a>(&'a self, key: &'a str) -> Option<&'a Self::Value<'a>> {
        self.get(key)
    }
}

impl<'ctx> JsonObjectLike for SimdObject<'ctx> {
    type Value<'a> = SimdBorrowedValue<'a> where 'ctx: 'a;

    fn get<'a>(&'a self, key: &'a str) -> Option<&'a Self::Value<'a>> {
        self.get(key)
    }
}
