pub use serde_json_borrow::*;

use super::model::{Children, ExecutionPlan, Field};
use super::store::Store;

#[allow(unused)]
pub struct Synth {
    operation: Vec<Field<Children>>,
    pub(crate) cache: Store,
}

#[allow(unused)]
impl Synth {
    pub fn new(operation: Vec<Field<Children>>) -> Self {
        Synth { operation, cache: Store::empty() }
    }
    fn build_children(
        &self,
        field: Field<Children>,
        query_blueprint: ExecutionPlan,
    ) -> ObjectAsVec {
        let mut object = vec![];
        for field in field.children() {
            let key = field.name.clone();
            let id = &field.id;
            if let Some(value) = self.cache.get(id) {
                object.push((key, value.get_value().to_owned()));
            }
        }
        object.into()
    }

    pub fn synthesize(&self) -> Value<'_> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use serde_json_borrow::OwnedValue;

    use crate::core::blueprint::Blueprint;
    use crate::core::config::reader::ConfigReader;
    use crate::core::ir::jit::builder::ExecutionPlanBuilder;
    use crate::core::ir::jit::model::FieldId;
    use crate::core::ir::jit::synth::Synth;

    #[tokio::test]
    async fn test_synth() {
        let rt = crate::core::runtime::test::init(None);
        let reader = ConfigReader::init(rt);
        let config = reader
            .read("examples/jsonplaceholder.graphql")
            .await
            .unwrap();
        let blueprint = Blueprint::try_from(&config).unwrap();
        let query = r#"
                query {
                    posts { user { name } }
                }
            "#;
        let document = async_graphql::parser::parse_query(query).unwrap();
        let q_blueprint = ExecutionPlanBuilder::new(blueprint)
            .build(document)
            .unwrap();
        let mut synth = Synth::new(q_blueprint.into_children());
        synth.cache.insert(
            FieldId::new(0),
            OwnedValue::parse_from(r#"[{"user":{"id":1,"name":"Leanne Graham"}}]"#.to_string())
                .unwrap(),
        );
    }
}
