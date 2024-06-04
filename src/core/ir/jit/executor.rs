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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::blueprint::Blueprint;
    use crate::core::config::Config;
    use crate::core::ir::jit::model::ExecutionPlanBuilder;
    use crate::core::valid::Validator;

    const CONFIG: &str = include_str!("./fixtures/jsonplaceholder-mutation.graphql");

    fn create_query_plan(query: impl AsRef<str>) -> ExecutionPlan {
        let config = Config::from_sdl(CONFIG).to_result().unwrap();
        let blueprint = Blueprint::try_from(&config.into()).unwrap();
        let document = async_graphql::parser::parse_query(query).unwrap();

        ExecutionPlanBuilder::new(blueprint)
            .build(document)
            .unwrap()
    }

    #[tokio::test]
    async fn test_from_document() {
        let plan = create_query_plan(
            r#"
            query {
                posts { user { id name } }
            }
        "#,
        );
        insta::assert_debug_snapshot!(plan);
    }

    #[test]
    fn test_simple_query() {
        let plan = create_query_plan(
            r#"
            query {
                posts { user { id } }
            }
        "#,
        );
        insta::assert_debug_snapshot!(plan);
    }

    #[test]
    fn test_simple_mutation() {
        let plan = create_query_plan(
            r#"
            mutation {
              createUser(user: {
                id: 101,
                name: "Tailcall",
                email: "tailcall@tailcall.run",
                phone: "2345234234",
                username: "tailcall",
                website: "tailcall.run"
              }) {
                id
                name
                email
                phone
                website
                username
              }
            }
        "#,
        );
        insta::assert_debug_snapshot!(plan);
    }

    #[test]
    fn test_fragments() {
        let plan = create_query_plan(
            r#"
            fragment UserPII on User {
              name
              email
              phone
            }

            query {
              user(id:1) {
                ...UserPII
              }
            }
        "#,
        );
        insta::assert_debug_snapshot!(plan);
    }

    #[test]
    fn test_multiple_operations() {
        let plan = create_query_plan(
            r#"
            query {
              user(id:1) {
                id
                username
              }
              posts {
                id
                title
              }
            }
        "#,
        );
        insta::assert_debug_snapshot!(plan);
    }

    #[test]
    fn test_variables() {
        let plan = create_query_plan(
            r#"
            query user($id: Int!) {
              user(id: $id) {
                id
                name
              }
            }
        "#,
        );
        insta::assert_debug_snapshot!(plan);
    }

    #[test]
    fn test_unions() {
        let plan = create_query_plan(
            r#"
            query {
              getUserIdOrEmail(id:1) {
                ...on UserId {
                  id
                }
                ...on UserEmail {
                  email
                }
              }
            }
        "#,
        );
        insta::assert_debug_snapshot!(plan);
    }

    #[test]
    fn test_default_value() {
        let plan = create_query_plan(
            r#"
            mutation {
              createPost(post:{
                userId:123,
                title:"tailcall",
                body:"tailcall test"
              }) {
                id
              }
            }
        "#,
        );
        insta::assert_debug_snapshot!(plan);
    }
}
