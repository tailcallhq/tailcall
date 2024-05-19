use std::collections::HashMap;

use serde::de::DeserializeSeed;
use serde_json::de::StrRead;

use crate::de::Deserialize;

#[derive(Debug)]
pub enum Schema {
    String,
    Number(N),
    Boolean,
    Object(HashMap<String, Box<Schema>>),
    Array(Box<Schema>),
}

#[derive(Debug)]
pub enum N {
    I64,
    U64,
    F64,
}

impl Schema {
    pub fn from_str(&self, input: &str) -> serde_json::Result<serde_json::Value> {
        let mut deserializer = serde_json::Deserializer::new(StrRead::new(input));
        Deserialize::new(self).deserialize(&mut deserializer)
    }

    pub fn array(item: Schema) -> Schema {
        Schema::Array(Box::new(item))
    }

    pub fn i64() -> Schema {
        Schema::Number(N::I64)
    }

    pub fn u64() -> Schema {
        Schema::Number(N::U64)
    }

    pub fn f64() -> Schema {
        Schema::Number(N::F64)
    }

    pub fn object(map: HashMap<String, Schema>) -> Schema {
        Schema::Object(map.into_iter().map(|(k, v)| (k, Box::new(v))).collect())
    }
}
