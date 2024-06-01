use crate::core::{BorrowedValue, extend_lifetime_ref};
use crate::core::mustache::Mustache;

#[derive(Debug, Clone)]
pub enum DynamicValue {
    Value(BorrowedValue),
    Mustache(Mustache),
    Object(Vec<(String, DynamicValue)>),
    Array(Vec<DynamicValue>),
}

impl TryFrom<&DynamicValue> for BorrowedValue {
    type Error = anyhow::Error;

    fn try_from(value: &DynamicValue) -> Result<Self, Self::Error> {
        match value {
            DynamicValue::Value(v) => Ok(v.to_owned()),
            DynamicValue::Mustache(_) => Err(anyhow::anyhow!(
                "mustache cannot be converted to const value"
            )),
            DynamicValue::Object(obj) => {
                let out: Result<Vec<(String, BorrowedValue)>, anyhow::Error> = obj
                    .into_iter()
                    .map(|(k, v)| Ok((k.clone(), BorrowedValue::try_from(v)?)))
                    .collect();
                Ok(BorrowedValue::Object(out?.into()))
            }
            DynamicValue::Array(arr) => {
                let out: Result<Vec<BorrowedValue>, anyhow::Error> =
                    arr.iter().map(BorrowedValue::try_from).collect();
                Ok(BorrowedValue::Array(out?))
            }
        }
    }
}

impl DynamicValue {
    // Helper method to determine if the value is constant (non-mustache).
    pub fn is_const(&self) -> bool {
        match self {
            DynamicValue::Mustache(m) => m.is_const(),
            DynamicValue::Object(obj) => obj.iter().all(|(_,v)| v.is_const()),
            DynamicValue::Array(arr) => arr.iter().all(|v| v.is_const()),
            _ => true,
        }
    }
}

impl TryFrom<&BorrowedValue> for DynamicValue {
    type Error = anyhow::Error;

    fn try_from(value: &BorrowedValue) -> Result<Self, Self::Error> {
        match value {
            BorrowedValue::Object(obj) => {
                let mut out = Vec::with_capacity(obj.len());
                for (k, v) in obj.iter().map(|(k,v)| (k, extend_lifetime_ref(v))) {
                    let dynamic_value = DynamicValue::try_from(v)?;
                    out.push((k.to_string(), dynamic_value));
                }
                Ok(DynamicValue::Object(out))
            }
            BorrowedValue::Array(arr) => {
                let out: Result<Vec<DynamicValue>, Self::Error> =
                    arr.iter().map(DynamicValue::try_from).collect();
                Ok(DynamicValue::Array(out?))
            }
            BorrowedValue::Str(s) => {
                let m = Mustache::parse(s)?;
                if m.is_const() {
                    Ok(DynamicValue::Value(value.clone().into()))
                } else {
                    Ok(DynamicValue::Mustache(m))
                }
            }
            _ => Ok(DynamicValue::Value(value.clone().into())),
        }
    }
}
