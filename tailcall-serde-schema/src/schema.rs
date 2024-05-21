use std::fmt::Display;

use fxhash::FxHashMap;
use serde::de::DeserializeSeed;
use serde_json::de::StrRead;

use crate::de::Value;

type Output<'de> = crate::value::Value<'de>;

#[derive(Debug, Clone)]
pub enum Schema {
    Primitive(Primitive),
    Object(FxHashMap<String, Schema>),
    Table {
        map: FxHashMap<String, Schema>,
        // Just a copy of the keys in row
        // Duplicated for performance reasons
        // TODO: could this be avoided somehow?
        head: Vec<String>,
        // Just a copy of the value in row
        // Duplicated for performance reasons
        // TODO: could this be avoided somehow?
        rows: Vec<Schema>,
    },
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

pub struct Owned {
    #[allow(dead_code)]
    input: String,
    value: Output<'static>,
}

impl Display for Owned {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}

fn extend_lifetime<'a>(s: Output<'a>) -> Output<'static> {
    unsafe { std::mem::transmute(s) }
}

impl Schema {
    pub fn from_str(&self, input: &str) -> serde_json::Result<Owned> {
        self.from_string(input.to_owned())
    }

    pub fn from_string(&self, input: String) -> serde_json::Result<Owned> {
        let mut deserializer = serde_json::Deserializer::new(StrRead::new(input.as_str()));
        let value = Value::new(self).deserialize(&mut deserializer)?;
        let value: Output<'static> = extend_lifetime(value);
        Ok(Owned { value, input })
    }

    pub fn table(schema: &[(&str, Schema)]) -> Self {
        Schema::Table {
            head: schema
                .iter()
                .map(|(k, _)| k.to_string())
                .collect::<Vec<_>>(),
            rows: schema.iter().map(|(_, v)| v.clone()).collect::<Vec<_>>(),
            map: schema
                .iter()
                .map(|(k, v)| (k.to_string(), v.clone()))
                .collect::<FxHashMap<_, _>>(),
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

    pub fn object(map: &[(&str, Schema)]) -> Self {
        Schema::Object(
            map.iter()
                .map(|(k, v)| (k.to_string(), v.to_owned()))
                .collect::<FxHashMap<_, _>>(),
        )
    }

    pub fn boolean() -> Self {
        Schema::Primitive(Primitive::Boolean)
    }

    pub fn string() -> Self {
        Schema::Primitive(Primitive::String)
    }
}

impl From<Primitive> for Schema {
    fn from(value: Primitive) -> Self {
        Schema::Primitive(value)
    }
}
