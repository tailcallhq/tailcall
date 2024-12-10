use async_graphql_value::{ConstValue, Name};
use indexmap::IndexMap;
use serde_json::Value;

use crate::core::mustache::JqTemplate;

#[derive(Debug, Clone, PartialEq)]
/// This is used to express dynamic value resolver engine.
pub enum DynamicValue<A> {
    Value(A),
    JqTemplate(JqTemplate),
    Object(IndexMap<Name, DynamicValue<A>>),
    Array(Vec<DynamicValue<A>>),
}

impl<A: Default> Default for DynamicValue<A> {
    fn default() -> Self {
        DynamicValue::Value(A::default())
    }
}

impl<A> DynamicValue<A> {
    /// This function is used to prepend a string to every Mustache Expression.
    /// This is useful when we want to hide a Mustache data argument from the
    /// user and make the use of Tailcall easier
    pub fn prepend(self, name: &str) -> Self {
        match self {
            DynamicValue::Value(value) => DynamicValue::Value(value),
            DynamicValue::JqTemplate(jqt) => DynamicValue::JqTemplate(JqTemplate(
                jqt.0
                    .into_iter()
                    .map(|mut f| match &mut f {
                        crate::core::mustache::JqTemplateIR::JqTransform(_) => f,
                        crate::core::mustache::JqTemplateIR::Literal(_) => f,
                        // this function can prepend a custom prefix to mustache only
                        crate::core::mustache::JqTemplateIR::Mustache(mustache) => {
                            let segments = mustache.segments_mut();
                            if let Some(crate::core::mustache::Segment::Expression(vec)) =
                                segments.get_mut(0)
                            {
                                vec.insert(0, name.to_string());
                            }
                            f
                        }
                    })
                    .collect(),
            )),
            DynamicValue::Object(index_map) => {
                let index_map = index_map
                    .into_iter()
                    .map(|(key, val)| (key, val.prepend(name)))
                    .collect();
                DynamicValue::Object(index_map)
            }
            DynamicValue::Array(vec) => {
                let vec = vec.into_iter().map(|val| val.prepend(name)).collect();
                DynamicValue::Array(vec)
            }
        }
    }
}

impl TryFrom<&DynamicValue<ConstValue>> for ConstValue {
    type Error = anyhow::Error;

    fn try_from(value: &DynamicValue<ConstValue>) -> Result<Self, Self::Error> {
        match value {
            DynamicValue::Value(v) => Ok(v.to_owned()),
            DynamicValue::JqTemplate(_) => Err(anyhow::anyhow!(
                "jq template cannot be converted to const value"
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

impl<A> DynamicValue<A> {
    // Helper method to determine if the value is constant (non-mustache).
    pub fn is_const(&self) -> bool {
        match self {
            DynamicValue::JqTemplate(t) => t.is_const(),
            DynamicValue::Object(obj) => obj.values().all(|v| v.is_const()),
            DynamicValue::Array(arr) => arr.iter().all(|v| v.is_const()),
            _ => true,
        }
    }
}

impl TryFrom<&Value> for DynamicValue<ConstValue> {
    type Error = anyhow::Error;

    /// Used to convert json notation to dynamic value
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
                let jqt = JqTemplate::parse(s);

                if jqt.is_const() {
                    Ok(DynamicValue::Value(ConstValue::from_json(value.clone())?))
                } else {
                    Ok(DynamicValue::JqTemplate(jqt))
                }
            }
            _ => Ok(DynamicValue::Value(ConstValue::from_json(value.clone())?)),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_dynamic_value_inject() {
        let value: DynamicValue<ConstValue> =
            DynamicValue::JqTemplate(JqTemplate::parse("{{.foo}}")).prepend("args");
        let expected: DynamicValue<ConstValue> =
            DynamicValue::JqTemplate(JqTemplate::parse("{{.args.foo}}"));
        assert_eq!(value, expected);

        let mut value_map = IndexMap::new();
        value_map.insert(
            Name::new("foo"),
            DynamicValue::JqTemplate(JqTemplate::parse("{{.foo}}")),
        );
        let value: DynamicValue<ConstValue> = DynamicValue::Object(value_map).prepend("args");
        let mut expected_map = IndexMap::new();
        expected_map.insert(
            Name::new("foo"),
            DynamicValue::JqTemplate(JqTemplate::parse("{{.args.foo}}")),
        );
        let expected: DynamicValue<ConstValue> = DynamicValue::Object(expected_map);
        assert_eq!(value, expected);

        let value: DynamicValue<ConstValue> = DynamicValue::Array(vec![DynamicValue::JqTemplate(
            JqTemplate::parse("{{.foo}}"),
        )])
        .prepend("args");
        let expected: DynamicValue<ConstValue> =
            DynamicValue::Array(vec![DynamicValue::JqTemplate(JqTemplate::parse(
                "{{.args.foo}}",
            ))]);
        assert_eq!(value, expected);

        let value: DynamicValue<ConstValue> = DynamicValue::Value(ConstValue::Null).prepend("args");
        let expected: DynamicValue<ConstValue> = DynamicValue::Value(ConstValue::Null);
        assert_eq!(value, expected);
    }
}
