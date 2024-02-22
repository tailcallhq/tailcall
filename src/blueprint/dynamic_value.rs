use async_graphql_value::Name;
use indexmap::IndexMap;
use serde_json::Value;

use crate::blueprint::MustacheOrValue;
use crate::mustache::Mustache;

#[derive(Debug, Clone)]
pub enum DynamicValue {
    Value(serde_json::Value),
    Mustache(Mustache),
    MustacheObject(IndexMap<Name, MustacheOrValue>),
    MustacheArray(Vec<MustacheOrValue>),
}

impl TryFrom<&serde_json::Value> for DynamicValue {
    type Error = anyhow::Error;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::Object(obj) => {
                let mut out = IndexMap::new();
                for (k, v) in obj {
                    let m = Mustache::parse(v.to_string().as_str())?;
                    if m.is_const() {
                        out.insert(Name::new(k), MustacheOrValue::Value(v.clone()));
                    } else {
                        out.insert(Name::new(k), MustacheOrValue::Mustache(m));
                    }
                }
                Ok(DynamicValue::MustacheObject(out))
            }
            Value::Array(arr) => {
                let mut out = Vec::new();
                for v in arr {
                    let m = Mustache::parse(v.to_string().as_str())?;
                    if m.is_const() {
                        out.push(MustacheOrValue::Value(v.clone()));
                    } else {
                        out.push(MustacheOrValue::Mustache(m));
                    }
                }
                Ok(DynamicValue::MustacheArray(out))
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
