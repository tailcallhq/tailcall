use std::fmt::Display;

#[derive(Debug)]
pub enum Value {
    Primitive(Primitive),
    Table {
        head: Vec<String>,
        rows: Vec<Vec<Value>>,
    },
    Array(Vec<Primitive>),
    Object(Vec<(String, Value)>),
}

#[derive(Debug)]
pub enum Primitive {
    Bool(bool),
    Number(N),
    String(String),
}

impl Primitive {
    pub fn from_i64(v: i64) -> Self {
        Primitive::Number(N::I64(v))
    }

    pub fn from_u64(v: u64) -> Self {
        Primitive::Number(N::U64(v))
    }

    pub fn from_f64(v: f64) -> Self {
        Primitive::Number(N::F64(v))
    }

    pub fn from_string(v: String) -> Self {
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

impl Value {
    pub fn from_i64(v: i64) -> Self {
        Value::Primitive(Primitive::Number(N::I64(v)))
    }

    pub fn from_u64(v: u64) -> Self {
        Value::Primitive(Primitive::Number(N::U64(v)))
    }

    pub fn from_f64(v: f64) -> Self {
        Value::Primitive(Primitive::Number(N::F64(v)))
    }

    pub fn from_string(v: String) -> Self {
        Value::Primitive(Primitive::String(v))
    }

    pub fn from_bool(v: bool) -> Self {
        Value::Primitive(Primitive::Bool(v))
    }

    pub fn from_array(v: Vec<Primitive>) -> Self {
        Value::Array(v)
    }

    pub fn from_object(v: Vec<(String, Value)>) -> Self {
        Value::Object(v)
    }

    pub fn from_table(head: Vec<String>, rows: Vec<Vec<Value>>) -> Self {
        Value::Table { head, rows }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("{:?}", self).as_str())
    }
}
