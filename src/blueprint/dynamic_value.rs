use async_graphql_value::{ConstValue, Name};
use indexmap::IndexMap;
use serde_json::Value;

use crate::mustache::Mustache;

#[derive(Debug, Clone)]
pub enum DynamicValue {
    Value(Value),
    Mustache(Mustache),
    Object(IndexMap<Name, DynamicValue>),
    Array(Vec<DynamicValue>),
}

impl TryFrom<&DynamicValue> for ConstValue {
    type Error = anyhow::Error;

    fn try_from(value: &DynamicValue) -> Result<Self, Self::Error> {
        match value {
            DynamicValue::Value(v) => {
                ConstValue::from_json(v.to_owned()).map_err(anyhow::Error::new)
            }
            DynamicValue::Mustache(_) => Err(anyhow::anyhow!(
                "mustache cannot be converted to const value at compiletime"
            )),
            DynamicValue::Object(obj) => {
                let out: Result<IndexMap<Name, ConstValue>, anyhow::Error> = obj
                    .into_iter()
                    .map(|(k, v)| Ok((k.to_owned(), ConstValue::try_from(v)?.to_owned())))
                    .collect();
                Ok(ConstValue::Object(out?))
            }
            DynamicValue::Array(arr) => {
                let out: Result<Vec<ConstValue>, anyhow::Error> =
                    arr.iter().map(ConstValue::try_from).collect();
                Ok(ConstValue::List(out?))
            }
        }
    }
}

impl DynamicValue {
    // Helper method to determine if the value is constant (non-mustache).
    pub fn is_const(&self) -> bool {
        match self {
            DynamicValue::Mustache(m) => m.is_const(),
            DynamicValue::Object(obj) => obj.values().all(|v| v.is_const()),
            DynamicValue::Array(arr) => arr.iter().all(|v| v.is_const()),
            _ => true,
        }
    }
}

impl TryFrom<&Value> for DynamicValue {
    type Error = anyhow::Error;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::Object(obj) => {
                let mut out = IndexMap::new();
                for (k, v) in obj {
                    let dynamic_value = DynamicValue::try_from(v)?;
                    out.insert(Name::new(k), dynamic_value);
                }
                Ok(DynamicValue::Object(out))
            }
            Value::Array(arr) => {
                let out: Result<Vec<DynamicValue>, Self::Error> =
                    arr.iter().map(DynamicValue::try_from).collect();
                Ok(DynamicValue::Array(out?))
            }
            Value::String(s) => {
                let m = Mustache::parse(s.as_str())?;
                if m.is_const() {
                    Ok(DynamicValue::Value(value.clone()))
                } else {
                    Ok(DynamicValue::Mustache(m))
                }
            }
            _ => Ok(DynamicValue::Value(value.clone())),
        }
    }
}
