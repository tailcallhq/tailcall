use serde::de::DeserializeSeed;
use serde_json::de::StrRead;

use crate::{de::Deserialize, value};

#[derive(Debug, Clone)]
pub enum Schema {
    Primitive(Primitive),
    Object(Vec<(String, Box<Schema>)>),
    Table { head: Vec<String>, row: Vec<Schema> },
    Array(Primitive),
}

#[derive(Debug, Clone)]
pub enum Primitive {
    Boolean,
    Number(N),
    String,
}

impl Primitive {
    pub fn boolean() -> Self {
        Primitive::Boolean
    }

    pub fn u64() -> Self {
        Primitive::Number(N::U64)
    }

    pub fn i64() -> Self {
        Primitive::Number(N::I64)
    }

    pub fn f64() -> Self {
        Primitive::Number(N::F64)
    }

    pub fn string() -> Self {
        Primitive::String
    }
}

#[derive(Debug, Clone)]
pub enum N {
    I64,
    U64,
    F64,
}

impl Schema {
    pub fn from_str(&self, input: &str) -> serde_json::Result<value::Value> {
        let mut deserializer = serde_json::Deserializer::new(StrRead::new(input));
        Deserialize::new(self).deserialize(&mut deserializer)
    }

    pub fn table(headers: &[&str], body: &[Schema]) -> Self {
        Schema::Table {
            head: headers.iter().map(|a| a.to_string()).collect::<Vec<_>>(),
            row: body.to_vec(),
        }
    }

    pub fn array(inner: Primitive) -> Self {
        Schema::Array(inner)
    }

    pub fn i64() -> Self {
        Schema::Primitive(Primitive::Number(N::I64))
    }

    pub fn u64() -> Self {
        Schema::Primitive(Primitive::Number(N::U64))
    }

    pub fn f64() -> Self {
        Schema::Primitive(Primitive::Number(N::F64))
    }

    pub fn object(map: Vec<(&str, Schema)>) -> Self {
        Schema::Object(
            map.into_iter()
                .map(|(k, v)| (k.to_string(), Box::new(v)))
                .collect::<Vec<_>>(),
        )
    }

    pub fn boolean() -> Self {
        Schema::Primitive(Primitive::Boolean)
    }

    pub fn string() -> Self {
        Schema::Primitive(Primitive::String)
    }
}

impl From<&Primitive> for Schema {
    fn from(value: &Primitive) -> Self {
        Schema::Primitive(value.to_owned())
    }
}
