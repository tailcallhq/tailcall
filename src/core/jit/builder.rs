use std::collections::HashMap;

use async_graphql::parser::types::{
    DocumentOperations, ExecutableDocument, FragmentDefinition, OperationType, Selection,
    SelectionSet,
};
use async_graphql::Positioned;
use async_graphql_value::Value;

use super::model::*;
use super::BuildError;
use crate::core::blueprint::{Blueprint, Index, QueryField};
use crate::core::counter::{Count, Counter};
use crate::core::jit::model::ExecutionPlan;
use crate::core::merge_right::MergeRight;

pub struct Builder {
    pub index: Index,
    pub arg_id: Counter<usize>,
    pub field_id: Counter<usize>,
    pub document: ExecutableDocument,
}

// TODO: make generic over Value (Input) type
impl Builder {
    pub fn new(blueprint: &Blueprint, document: ExecutableDocument) -> Self {
        let index = blueprint.index();
        Self {
            document,
            index,
            arg_id: Counter::default(),
            field_id: Counter::default(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn iter(
        &self,
        selection: &SelectionSet,
        type_of: &str,
        exts: Option<Flat>,
        fragments: &HashMap<&str, &FragmentDefinition>,
    ) -> Vec<Field<Flat, Value>> {
        let mut fields = vec![];
        for selection in &selection.items {
            match &selection.node {
                Selection::Field(Positioned { node: gql_field, .. }) => {
                    let field_name = gql_field.name.node.as_str();
                    let request_args = gql_field
                        .arguments
                        .iter()
                        .map(|(k, v)| (k.node.as_str().to_string(), v.node.to_owned()))
                        .collect::<HashMap<_, _>>();

                    if let Some(field_def) = self.index.get_field(type_of, field_name) {
                        let mut args = Vec::with_capacity(request_args.len());
                        if let QueryField::Field((_, schema_args)) = field_def {
                            for (arg_name, arg_value) in schema_args {
                                let type_of = arg_value.of_type.clone();
                                let id = ArgId::new(self.arg_id.next());
                                let name = arg_name.clone();
                                let default_value = arg_value
                                    .default_value
                                    .as_ref()
                                    .and_then(|v| v.to_owned().try_into().ok());
                                args.push(Arg {
                                    id,
                                    name,
                                    type_of,
                                    // TODO: handle errors for non existing request_args without the
                                    // default
                                    value: request_args.get(arg_name).cloned(),
                                    default_value,
                                });
                            }
                        }

                        let type_of = match field_def {
                            QueryField::Field((field_def, _)) => field_def.of_type.clone(),
                            QueryField::InputField(field_def) => field_def.of_type.clone(),
                        };

                        let id = FieldId::new(self.field_id.next());
                        let child_fields = self.iter(
                            &gql_field.selection_set.node,
                            type_of.name(),
                            Some(Flat::new(id.clone())),
                            fragments,
                        );
                        let name = field_name.to_owned();
                        let ir = match field_def {
                            QueryField::Field((field_def, _)) => field_def.resolver.clone(),
                            _ => None,
                        };
                        fields.push(Field {
                            id,
                            name,
                            ir,
                            type_of,
                            args,
                            extensions: exts.clone(),
                        });
                        fields = fields.merge_right(child_fields);
                    } else {
                        // TODO: error if the field is not found in the schema
                    }
                }
                Selection::FragmentSpread(Positioned { node: fragment_spread, .. }) => {
                    if let Some(fragment) =
                        fragments.get(fragment_spread.fragment_name.node.as_str())
                    {
                        fields.extend(self.iter(
                            &fragment.selection_set.node,
                            fragment.type_condition.node.on.node.as_str(),
                            exts.clone(),
                            fragments,
                        ));
                    }
                }
                _ => {}
            }
        }

        fields
    }

    fn get_type(&self, ty: OperationType) -> Option<&str> {
        match ty {
            OperationType::Query => Some(self.index.get_query()),
            OperationType::Mutation => self.index.get_mutation(),
            OperationType::Subscription => None,
        }
    }

    pub fn build(&self) -> Result<ExecutionPlan<Value>, BuildError> {
        let mut fields = Vec::new();
        let mut fragments: HashMap<&str, &FragmentDefinition> = HashMap::new();

        for (name, fragment) in self.document.fragments.iter() {
            fragments.insert(name.as_str(), &fragment.node);
        }

        match &self.document.operations {
            DocumentOperations::Single(single) => {
                let name = self
                    .get_type(single.node.ty)
                    .ok_or(BuildError::RootOperationTypeNotDefined { operation: single.node.ty })?;
                fields.extend(self.iter(&single.node.selection_set.node, name, None, &fragments));
            }
            DocumentOperations::Multiple(multiple) => {
                for single in multiple.values() {
                    let name = self.get_type(single.node.ty).ok_or(
                        BuildError::RootOperationTypeNotDefined { operation: single.node.ty },
                    )?;
                    fields.extend(self.iter(
                        &single.node.selection_set.node,
                        name,
                        None,
                        &fragments,
                    ));
                }
            }
        }

        Ok(ExecutionPlan::new(fields))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::core::blueprint::Blueprint;
    use crate::core::config::Config;
    use crate::core::jit::builder::Builder;
    use crate::core::valid::Validator;

    const CONFIG: &str = include_str!("./fixtures/jsonplaceholder-mutation.graphql");

    fn plan(query: impl AsRef<str>) -> ExecutionPlan<Value> {
        let config = Config::from_sdl(CONFIG).to_result().unwrap();
        let blueprint = Blueprint::try_from(&config.into()).unwrap();
        let document = async_graphql::parser::parse_query(query).unwrap();

        Builder::new(&blueprint, document).build().unwrap()
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
        insta::assert_debug_snapshot!(plan.into_nested());
    }

    #[tokio::test]
    async fn test_size() {
        let plan = plan(
            r#"
            query {
                posts { user { id name } }
            }
        "#,
        );

        assert_eq!(plan.size(), 4)
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
        insta::assert_debug_snapshot!(plan.into_nested());
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
        insta::assert_debug_snapshot!(plan.into_nested());
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
        insta::assert_debug_snapshot!(plan.into_nested());
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
        insta::assert_debug_snapshot!(plan.into_nested());
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
        insta::assert_debug_snapshot!(plan.into_nested());
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
        insta::assert_debug_snapshot!(plan.into_nested());
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
        insta::assert_debug_snapshot!(plan.into_nested());
    }
}
