use futures_util::future;
use serde_json_borrow::OwnedValue;

use super::model::{ExecutionPlan, Field, FieldId, Parent};
use super::store::Store;
use crate::core::ir::IR;

#[allow(unused)]
pub struct ExecutionContext {
    plan: ExecutionPlan,
    cache: Store,
}

#[allow(unused)]
impl ExecutionContext {
    pub async fn execute_ir(
        &self,
        ir: &IR,
        parent: Option<&OwnedValue>,
    ) -> anyhow::Result<OwnedValue> {
        todo!()
    }
    fn find_children(&self, id: FieldId) -> Vec<Field<Parent>> {
        todo!()
    }

    fn insert_field_value(&self, id: FieldId, value: OwnedValue) {
        todo!()
    }

    fn find_field(&self, id: FieldId) -> Option<&Field<Parent>> {
        self.plan.fields.iter().find(|field| field.id == id)
    }

    async fn execute_field(&self, id: FieldId, parent: Option<&OwnedValue>) -> anyhow::Result<()> {
        if let Some(field) = self.find_field(id.clone()) {
            if let Some(ir) = &field.ir {
                let value = self.execute_ir(ir, parent).await?;

                let children = self.find_children(id.clone());
                future::join_all(
                    children
                        .into_iter()
                        .map(|child| self.execute_field(child.id, Some(&value))),
                )
                .await
                .into_iter()
                .collect::<anyhow::Result<Vec<_>>>()?;

                self.insert_field_value(id, value);
            }
        }
        Ok(())
    }

    fn root(&self) -> Vec<&Field<Parent>> {
        self.plan
            .fields
            .iter()
            .filter(|field| field.refs.is_none())
            .collect::<Vec<_>>()
    }

    pub async fn execute(&self) -> anyhow::Result<()> {
        future::join_all(
            self.root()
                .iter()
                .map(|field| self.execute_field(field.id.to_owned(), None)),
        )
        .await
        .into_iter()
        .collect::<anyhow::Result<Vec<_>>>()?;
        Ok(())
    }
}
