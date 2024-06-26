use async_graphql_value::{ConstValue, Name};
use indexmap::IndexMap;
use serde_json::Value;

use crate::core::error::Error;
use crate::core::mustache::Mustache;

#[derive(Debug, Clone)]
pub enum DynamicValue<A> {
    Value(A),
    Mustache(Mustache),
    Object(IndexMap<Name, DynamicValue<A>>),
    Array(Vec<DynamicValue<A>>),
}

impl TryFrom<&DynamicValue<ConstValue>> for ConstValue {
    type Error = Error;

    fn try_from(value: &DynamicValue<ConstValue>) -> Result<Self, Self::Error> {
        match value {
            DynamicValue::Value(v) => Ok(v.to_owned()),
            DynamicValue::Mustache(_) => Err(Error::InvalidMustacheConstConversion),
            DynamicValue::Object(obj) => {
                let out: Result<IndexMap<Name, ConstValue>, Error> = obj
                    .into_iter()
                    .map(|(k, v)| Ok((k.to_owned(), ConstValue::try_from(v)?.to_owned())))
                    .collect();
                Ok(ConstValue::Object(out?))
            }
            DynamicValue::Array(arr) => {
                let out: Result<Vec<ConstValue>, Error> =
                    arr.iter().map(ConstValue::try_from).collect();
                Ok(ConstValue::List(out?))
            }
        }
    }
}

impl<A> DynamicValue<A> {
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

impl TryFrom<&Value> for DynamicValue<ConstValue> {
    type Error = Error;

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
                let out: Result<Vec<DynamicValue<ConstValue>>, Self::Error> =
                    arr.iter().map(DynamicValue::try_from).collect();
                Ok(DynamicValue::Array(out?))
            }
            Value::String(s) => {
                let m = Mustache::parse(s.as_str())?;
                if m.is_const() {
                    Ok(DynamicValue::Value(ConstValue::from_json(value.clone())?))
                } else {
                    Ok(DynamicValue::Mustache(m))
                }
            }
            _ => Ok(DynamicValue::Value(ConstValue::from_json(value.clone())?)),
        }
    }
}

impl<'a> From<&'a DynamicValue<serde_json::Value>> for serde_json_borrow::Value<'a> {
    fn from(_value: &'a DynamicValue<serde_json::Value>) -> Self {
        todo!()
    }
}
