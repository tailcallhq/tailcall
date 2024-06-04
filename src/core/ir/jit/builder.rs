use std::collections::HashMap;

use async_graphql::parser::types::{DocumentOperations, ExecutableDocument, Selection};
use async_graphql_parser::types::SelectionSet;

use super::field_index::{FieldIndex, QueryField};
use super::model::*;
use crate::core::blueprint::Blueprint;
use crate::core::counter::Counter;
use crate::core::merge_right::MergeRight;

pub struct ExecutionPlanBuilder {
    index: FieldIndex,
    arg_id: Counter,
    field_id: Counter,
}

impl ExecutionPlanBuilder {
    #[allow(unused)]
    pub fn new(blueprint: Blueprint) -> Self {
        let blueprint_index = FieldIndex::init(&blueprint);
        Self {
            index: blueprint_index,
            arg_id: Counter::default(),
            field_id: Counter::default(),
        }
    }

    #[allow(unused)]
    pub fn build(&self, document: ExecutableDocument) -> anyhow::Result<ExecutionPlan> {
        let fields = self.create_field_set(document)?;
        Ok(ExecutionPlan { fields })
    }

    fn iter(
        &self,
        selection: SelectionSet,
        type_of: &str,
        parent: Option<Parent>,
    ) -> anyhow::Result<Vec<Field<Parent>>> {
        let mut fields = Vec::new();

        for selection in selection.items {
            if let Selection::Field(gql_field) = selection.node {
                let field_name = gql_field.node.name.node.as_str();
                let field_args = gql_field
                    .node
                    .arguments
                    .into_iter()
                    .map(|(k, v)| (k.node.as_str().to_string(), v.node))
                    .collect::<HashMap<_, _>>();

                if let Some(field_def) = self.index.get_field(type_of, field_name) {
                    let mut args = vec![];
                    for (arg_name, value) in field_args {
                        if let Some(arg) = field_def.get_arg(&arg_name) {
                            let type_of = arg.of_type.clone();
                            let id = ArgId::new(self.arg_id.next());
                            let arg = Arg {
                                id,
                                name: arg_name.clone(),
                                type_of,
                                value: Some(value),
                                default_value: arg
                                    .default_value
                                    .as_ref()
                                    .and_then(|v| v.to_owned().try_into().ok()),
                            };
                            args.push(arg);
                        }
                    }

                    let type_of = match field_def {
                        QueryField::Field((field_def, _)) => field_def.of_type.clone(),
                        QueryField::InputField(field_def) => field_def.of_type.clone(),
                    };

                    let cur_id = FieldId::new(self.field_id.next());
                    let child_fields = self.iter(
                        gql_field.node.selection_set.node.clone(),
                        type_of.name(),
                        Some(Parent::new(cur_id.clone())),
                    )?;
                    let field = Field {
                        id: cur_id,
                        name: field_name.to_string(),
                        ir: match field_def {
                            QueryField::Field((field_def, _)) => field_def.resolver.clone(),
                            _ => None,
                        },
                        type_of,
                        args,
                        refs: parent.clone(),
                    };

                    fields.push(field);
                    fields = fields.merge_right(child_fields);
                }
            }
        }

        Ok(fields)
    }

    fn create_field_set(&self, document: ExecutableDocument) -> anyhow::Result<Vec<Field<Parent>>> {
        let query = &self.index.get_query().to_owned();

        let mut fields = Vec::new();

        for (_, fragment) in document.fragments {
            fields = self.iter(fragment.node.selection_set.node, query, None)?;
        }

        match document.operations {
            DocumentOperations::Single(single) => {
                fields = self.iter(single.node.selection_set.node, query, None)?;
            }
            DocumentOperations::Multiple(multiple) => {
                for (_, single) in multiple {
                    fields = self.iter(single.node.selection_set.node, query, None)?;
                }
            }
        }

        Ok(fields)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::blueprint::Blueprint;
    use crate::core::config::Config;
    use crate::core::ir::jit::builder::ExecutionPlanBuilder;
    use crate::core::valid::Validator;

    const CONFIG: &str = include_str!("./fixtures/jsonplaceholder-mutation.graphql");

    fn plan(query: impl AsRef<str>) -> ExecutionPlan {
        let config = Config::from_sdl(CONFIG).to_result().unwrap();
        let blueprint = Blueprint::try_from(&config.into()).unwrap();
        let document = async_graphql::parser::parse_query(query).unwrap();

        ExecutionPlanBuilder::new(blueprint)
            .build(document)
            .unwrap()
    }

    #[tokio::test]
    async fn test_from_document() {
        let plan = plan(
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
        let plan = plan(
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
        let plan = plan(
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
        let plan = plan(
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
        let plan = plan(
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
        let plan = plan(
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
        let plan = plan(
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
        let plan = plan(
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
