use async_graphql_value::ConstValue;

pub trait JsonLike {
    type Output;
    fn as_array_ok(&self) -> Result<&Vec<Self::Output>, &str>;
    fn as_str_ok(&self) -> Result<&str, &str>;
    fn as_i64_ok(&self) -> Result<i64, &str>;
    fn as_u64_ok(&self) -> Result<u64, &str>;
    fn as_f64_ok(&self) -> Result<f64, &str>;
    fn as_bool_ok(&self) -> Result<bool, &str>;
    fn as_null_ok(&self) -> Result<(), &str>;
    fn as_option_ok(&self) -> Result<Option<&Self::Output>, &str>;
    fn get_path(&self, path: &[String]) -> Option<&Self::Output>;
    fn new(value: Self::Output) -> Self;
}

impl JsonLike for serde_json::Value {
    type Output = serde_json::Value;
    fn as_array_ok(&self) -> Result<&Vec<Self::Output>, &str> {
        self.as_array().ok_or("expected array")
    }
    fn as_str_ok(&self) -> Result<&str, &str> {
        self.as_str().ok_or("expected str")
    }
    fn as_i64_ok(&self) -> Result<i64, &str> {
        self.as_i64().ok_or("expected i64")
    }
    fn as_u64_ok(&self) -> Result<u64, &str> {
        self.as_u64().ok_or("expected u64")
    }
    fn as_f64_ok(&self) -> Result<f64, &str> {
        self.as_f64().ok_or("expected f64")
    }
    fn as_bool_ok(&self) -> Result<bool, &str> {
        self.as_bool().ok_or("expected bool")
    }
    fn as_null_ok(&self) -> Result<(), &str> {
        self.as_null().ok_or("expected null")
    }

    fn as_option_ok(&self) -> Result<Option<&Self::Output>, &str> {
        match self {
            serde_json::Value::Null => Ok(None),
            _ => Ok(Some(self)),
        }
    }

    fn get_path(&self, path: &[String]) -> Option<&Self::Output> {
        let mut val = self;
        for token in path {
            val = match val {
                serde_json::Value::Array(arr) => {
                    let index = token.parse::<usize>().ok()?;
                    arr.get(index)?
                }
                serde_json::Value::Object(map) => map.get(token)?,
                _ => return None,
            };
        }
        Some(val)
    }

    fn new(value: Self::Output) -> Self {
        value
    }
}

impl JsonLike for async_graphql::Value {
    type Output = async_graphql::Value;

    fn as_array_ok(&self) -> Result<&Vec<Self::Output>, &str> {
        match self {
            ConstValue::List(seq) => Ok(seq),
            _ => Err("array"),
        }
    }

    fn as_str_ok(&self) -> Result<&str, &str> {
        match self {
            ConstValue::String(s) => Ok(s),
            _ => Err("str"),
        }
    }

    fn as_i64_ok(&self) -> Result<i64, &str> {
        match self {
            ConstValue::Number(n) => n.as_i64().ok_or("expected i64"),
            _ => Err("i64"),
        }
    }

    fn as_u64_ok(&self) -> Result<u64, &str> {
        match self {
            ConstValue::Number(n) => n.as_u64().ok_or("expected u64"),
            _ => Err("u64"),
        }
    }

    fn as_f64_ok(&self) -> Result<f64, &str> {
        match self {
            ConstValue::Number(n) => n.as_f64().ok_or("expected f64"),
            _ => Err("f64"),
        }
    }

    fn as_bool_ok(&self) -> Result<bool, &str> {
        match self {
            ConstValue::Boolean(b) => Ok(*b),
            _ => Err("bool"),
        }
    }

    fn as_null_ok(&self) -> Result<(), &str> {
        match self {
            ConstValue::Null => Ok(()),
            _ => Err("null"),
        }
    }

    fn as_option_ok(&self) -> Result<Option<&Self::Output>, &str> {
        match self {
            ConstValue::Null => Ok(None),
            _ => Ok(Some(self)),
        }
    }

    fn get_path(&self, path: &[String]) -> Option<&Self::Output> {
        let mut val = self;
        for token in path {
            val = match val {
                ConstValue::List(seq) => {
                    let index = token.parse::<usize>().ok()?;
                    seq.get(index)?
                }
                ConstValue::Object(map) => map.get(&async_graphql::Name::new(token))?,
                _ => return None,
            };
        }
        Some(val)
    }

    fn new(value: Self::Output) -> Self {
        value
    }
}
