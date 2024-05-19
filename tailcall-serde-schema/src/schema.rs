use std::collections::HashMap;

use serde::de::DeserializeSeed;

use crate::de::{Deserialize, Deserializer};

pub enum Schema {
    String,
    Number(N),
    Boolean,
    Object(HashMap<String, Box<Schema>>),
    Array(Box<Schema>),
}
pub enum N {
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
    F32,
    F64,
}

impl Schema {
    pub fn deserialize(&self, input: &str) -> serde_json::Result<serde_json::Value> {
        Deserialize::new(self).deserialize(Deserializer::new(input))
    }
}
