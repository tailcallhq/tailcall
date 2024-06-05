pub use serde_json_borrow::*;

use super::model::{Children, ExecutionPlan, Field};
use super::store::Store;

#[allow(unused)]
pub struct Synth {
    operation: Field<Children>,
    pub(crate) store: Store,
}

#[allow(unused)]
impl Synth {
    pub fn new(operation: Field<Children>) -> Self {
        Synth { operation, store: Store::empty() }
    }
    fn build_children(
        &self,
        field: Field<Children>,
        query_blueprint: ExecutionPlan,
    ) -> ObjectAsVec {
        let mut object = vec![];
        for field in field.children() {
            let key = &field.name;
            let id = &field.id;
            if let Some(value) = self.store.get(id) {
                object.push((key.to_owned(), value.get_value().to_owned()));
            }
        }
        object.into()
    }

    pub fn synthesize(&self) -> Value<'_> {
        self.synthesize_internal(&self.operation, None)
    }
    fn validate(type_of: &crate::core::blueprint::Type, value: &Value) -> bool {
        type_of.is_list() == value.is_array()
    }
    fn synthesize_internal(&self, node: &Field<Children>, value: Option<&Value>) -> Value<'static> {
        if node.ir.is_some() {
            if let Some(value) = self.store.get(&node.id) {
                let value = value.get_value();
                if !Self::validate(&node.type_of, value) {
                    return Value::Null;
                }
                match value {
                    Value::Array(vals) => {
                        let mut vec = vec![];
                        for val in vals {
                            for child in node.children().iter() {
                                let val = self.synthesize_internal(child, Some(val));
                                vec.push(val);
                            }
                        }
                        Value::Array(vec)
                    }
                    val => {
                        let mut vec = vec![];
                        for child in node.children().iter() {
                            let val = self.synthesize_internal(child, Some(val));
                            vec.push(val);
                        }
                        Value::Array(vec)
                    }
                }
            } else {
                Value::Null
            }
        } else {
            if !Self::validate(&node.type_of, value.unwrap_or(&Value::Null)) {
                return Value::Null;
            }

            match value {
                Some(Value::Object(value)) => {
                    let result = value.iter().find(|(k, v)| *k == node.name);
                    let val = result.map(|v| v.1).unwrap_or(&Value::Null);
                    let children = node.children();
                    if children.is_empty() {
                        extend_lifetime(val.clone())
                    } else {
                        let mut vec = vec![];
                        for child in node.children() {
                            let val = self.synthesize_internal(child, Some(val));
                            vec.push((child.name.to_owned(), val));
                        }
                        Value::Object(vec.into())
                    }
                }
                Some(Value::Array(vals)) => {
                    let mut vec = vec![];
                    for val in vals {
                        for child in node.children().iter() {
                            let val = self.synthesize_internal(child, Some(val));
                            vec.push(val);
                        }
                    }
                    Value::Array(vec)
                }
                _ => Value::Null,
            }
        }
    }
}

fn extend_lifetime<'b>(r: Value<'b>) -> Value<'static> {
    unsafe { std::mem::transmute::<Value<'b>, Value<'static>>(r) }
}

#[cfg(test)]
mod tests {
    use serde_json_borrow::{Number, OwnedValue, Value};

    use crate::core::blueprint::Blueprint;
    use crate::core::config::reader::ConfigReader;
    use crate::core::ir::jit::model::{ExecutionPlanBuilder, FieldId};
    use crate::core::ir::jit::synth::Synth;

    async fn get_bp() -> Blueprint {
        let rt = crate::core::runtime::test::init(None);
        let reader = ConfigReader::init(rt);
        let config = reader
            .read(tailcall_fixtures::configs::synth::TEST_SYNTH)
            .await
            .unwrap();
        Blueprint::try_from(&config).unwrap()
    }

    #[tokio::test]
    async fn test_synth_with_empty_store() {
        let blueprint = get_bp().await;
        let query = r#"
                query {
                    posts { user { name } }
                }
            "#;
        let document = async_graphql::parser::parse_query(query).unwrap();
        let q_blueprint = ExecutionPlanBuilder::new(blueprint)
            .build(document)
            .unwrap();
        let children = q_blueprint.into_children();
        if let Some(child) = children.first() {
            let synth = Synth::new(child.clone());
            assert_eq!(synth.synthesize(), Value::Null);
        }
    }

    #[tokio::test]
    async fn test_synth_with_non_list_value() {
        let blueprint = get_bp().await;
        let query = r#"
                query {
                    posts { user { name } }
                }
            "#;
        let document = async_graphql::parser::parse_query(query).unwrap();
        let q_blueprint = ExecutionPlanBuilder::new(blueprint)
            .build(document)
            .unwrap();
        let children = q_blueprint.into_children();
        if let Some(child) = children.first() {
            let mut synth = Synth::new(child.clone());
            synth.store.map.push((
                FieldId::new(0),
                OwnedValue::parse_from(r#"{"user":{"id":1,"name":"Leanne Graham"}}"#.to_string())
                    .unwrap(),
            ));
            assert_eq!(synth.synthesize(), Value::Null);
        }
    }

    #[tokio::test]
    async fn test_synth_with_list_value() {
        let query = r#"
                query {
                    posts { user { name } }
                }
            "#;
        let document = async_graphql::parser::parse_query(query).unwrap();
        let blueprint = get_bp().await;

        let q_blueprint = ExecutionPlanBuilder::new(blueprint)
            .build(document)
            .unwrap();
        let children = q_blueprint.into_children();
        if let Some(child) = children.first() {
            let mut synth = Synth::new(child.clone());
            synth.store.map.push((
                FieldId::new(0),
                OwnedValue::parse_from(r#"[{"user":{"id":1,"name":"Leanne Graham"}}]"#.to_string())
                    .unwrap(),
            ));
            assert_eq!(
                synth.synthesize(),
                Value::Array(vec![Value::Object(
                    vec![("name".to_string(), Value::Str("Leanne Graham".into()))].into()
                )])
            );
        }
    }

    #[tokio::test]
    async fn test_synthesize_nested_object() {
        let query = r#"
                query {
                    users { id }
                }
            "#;
        let document = async_graphql::parser::parse_query(query).unwrap();
        let blueprint = get_bp().await;

        let q_blueprint = ExecutionPlanBuilder::new(blueprint)
            .build(document)
            .unwrap();
        let children = q_blueprint.into_children();
        if let Some(child) = children.first() {
            let mut synth = Synth::new(child.clone());
            synth.store.map.push((
                FieldId::new(0),
                OwnedValue::parse_from(r#"[{"id":1,"name":"Leanne Graham"}]"#.to_string()).unwrap(),
            ));
            assert_eq!(
                synth.synthesize(),
                Value::Array(vec![Value::Number(Number::from(1u64))])
            ); // TODO fix borrowed value's number handling
        }
    }
}
