use std::fmt::Display;

#[derive(Debug)]
pub enum Value<'de> {
    Primitive(Primitive<'de>),
    Table(Vec<Vec<Value<'de>>>),
    Array(Vec<Primitive<'de>>),
    Object(Vec<Value<'de>>),
}

#[derive(Debug)]
pub enum Primitive<'de> {
    Bool(bool),
    Number(N),
    String(&'de str),
}

impl<'de> Primitive<'de> {
    pub fn from_i64(v: i64) -> Self {
        Primitive::Number(N::I64(v))
    }

    pub fn from_u64(v: u64) -> Self {
        Primitive::Number(N::U64(v))
    }

    pub fn from_f64(v: f64) -> Self {
        Primitive::Number(N::F64(v))
    }

    pub fn from_str(v: &'de str) -> Self {
        Primitive::String(v)
    }

    pub fn from_bool(v: bool) -> Self {
        Primitive::Bool(v)
    }
}

#[derive(Debug)]
pub enum N {
    I64(i64),
    U64(u64),
    F64(f64),
}

impl<'de> Value<'de> {
    pub fn from_i64(v: i64) -> Self {
        Value::Primitive(Primitive::Number(N::I64(v)))
    }

    pub fn from_u64(v: u64) -> Self {
        Value::Primitive(Primitive::Number(N::U64(v)))
    }

    pub fn from_f64(v: f64) -> Self {
        Value::Primitive(Primitive::Number(N::F64(v)))
    }

    pub fn from_str(v: &'de str) -> Self {
        Value::Primitive(Primitive::String(v))
    }

    pub fn from_bool(v: bool) -> Self {
        Value::Primitive(Primitive::Bool(v))
    }

    pub fn from_array(v: Vec<Primitive<'de>>) -> Self {
        Value::Array(v)
    }

    pub fn from_object(v: Vec<Value<'de>>) -> Self {
        Value::Object(v)
    }

    pub fn from_table(rows: Vec<Vec<Value<'de>>>) -> Self {
        Value::Table(rows)
    }
}

impl<'de> Display for Value<'de> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("{:?}", self).as_str())
    }
}
