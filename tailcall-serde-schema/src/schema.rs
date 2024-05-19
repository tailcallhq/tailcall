use std::collections::HashMap;

use serde::de::DeserializeSeed;
use serde_json::de::StrRead;

use crate::de::Deserialize;

pub enum Schema {
    String,
    Number(N),
    Boolean,
    Object(HashMap<String, Box<Schema>>),
    Array(Box<Schema>),
}
pub enum N {
    I64,
    U64,
    F64,
}

impl Schema {
    pub fn deserialize(&self, input: &str) -> serde_json::Result<serde_json::Value> {
        let mut deserializer = serde_json::Deserializer::new(StrRead::new(input));
        Deserialize::new(self).deserialize(&mut deserializer)
    }

    pub fn array(item: Schema) -> Schema {
        Schema::Array(Box::new(item))
    }
}
