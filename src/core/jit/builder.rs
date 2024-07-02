use std::collections::HashMap;

use async_graphql::parser::types::{
    DocumentOperations, ExecutableDocument, OperationDefinition, OperationType, Selection,
    SelectionSet,
};
use async_graphql::Positioned;
use async_graphql_value::ConstValue;

use super::model::*;
use crate::core::blueprint::{Blueprint, Index, QueryField};
use crate::core::counter::{Count, Counter};
use crate::core::jit::model::ExecutionPlan;
use crate::core::merge_right::MergeRight;

pub trait Builder<ParsedValue: Clone, Input: Clone> {
    fn build(&self) -> Result<ExecutionPlan<ParsedValue, Input>, String>;
}

pub trait FromParsedValue<ParsedValue>
where
    Self: Sized,
{
    fn from_parsed_value(value: ParsedValue, variables: &HashMap<String, Self>) -> Option<Self>;
}

impl FromParsedValue<async_graphql_value::Value> for ConstValue {
    fn from_parsed_value(
        value: async_graphql_value::Value,
        variables: &HashMap<String, Self>,
    ) -> Option<Self> {
        value
            .into_const_with(|name| {
                variables
                    .get(name.as_str())
                    .cloned()
                    .ok_or("Variable not found".to_string())
            })
            .ok()
    }
}

pub struct ConstBuilder {
    pub index: Index,
    pub arg_id: Counter<usize>,
    pub field_id: Counter<usize>,
    pub document: ExecutableDocument,
    pub variables: Option<HashMap<String, ConstValue>>,
}

impl ConstBuilder {
    pub fn new(
        blueprint: &Blueprint,
        document: ExecutableDocument,
        variables: Option<HashMap<String, ConstValue>>,
    ) -> Self {
        let index = blueprint.index();
        Self {
            document,
            index,
            arg_id: Counter::default(),
            field_id: Counter::default(),
            variables,
        }
    }

    pub fn iter(
        &self,
        selection: &SelectionSet,
        type_of: &str,
        refs: Option<Parent>,
    ) -> Vec<Field<Parent, async_graphql_value::Value, async_graphql::Value>> {
        let mut fields = vec![];
        for selection in &selection.items {
            if let Selection::Field(gql_field) = &selection.node {
                let field_name = gql_field.node.name.node.as_str();
                let mut field_args = gql_field
                    .node
                    .arguments
                    .iter()
                    .map(|(k, v)| (k.node.as_str().to_string(), v.node.to_owned()))
                    .collect::<HashMap<_, _>>();

                if let Some(field_def) = self.index.get_field(type_of, field_name) {
                    if let QueryField::Field((_, args)) = field_def {
                        for (arg_name, arg_value) in args {
                            if let Some(default_value) = arg_value.default_value.as_ref() {
                                if !field_args.contains_key(arg_name) {
                                    if let Ok(default_value) =
                                        async_graphql_value::Value::from_json(default_value.clone())
                                    {
                                        field_args.insert(arg_name.clone(), default_value);
                                    }
                                }
                            }
                        }
                    }

                    let mut args = vec![];
                    for (arg_name, value) in field_args {
                        if let Some(arg) = field_def.get_arg(&arg_name) {
                            let type_of = arg.of_type.clone();
                            let id = ArgId::new(self.arg_id.next());
                            let name = arg_name.clone();
                            let default_value = arg
                                .default_value
                                .as_ref()
                                .and_then(|v| v.to_owned().try_into().ok());
                            args.push(Arg { id, name, type_of, value: Some(value), default_value });
                        }
                    }

                    let type_of = match field_def {
                        QueryField::Field((field_def, _)) => field_def.of_type.clone(),
                        QueryField::InputField(field_def) => field_def.of_type.clone(),
                    };

                    let id = FieldId::new(self.field_id.next());
                    let child_fields = self.iter(
                        &gql_field.node.selection_set.node,
                        type_of.name(),
                        Some(Parent::new(id.clone())),
                    );
                    let name = field_name.to_owned();
                    let ir = match field_def {
                        QueryField::Field((field_def, _)) => field_def.resolver.clone(),
                        _ => None,
                    };

                    fields.push(Field { id, name, ir, type_of, args, refs: refs.clone() });
                    fields = fields.merge_right(child_fields);
                }
            }
        }

        fields
    }

    pub fn get_type(&self, ty: OperationType) -> Option<&str> {
        match ty {
            OperationType::Query => Some(self.index.get_query()),
            OperationType::Mutation => self.index.get_mutation(),
            OperationType::Subscription => None,
        }
    }
}

impl Builder<async_graphql_value::Value, ConstValue> for ConstBuilder {
    fn build(&self) -> Result<ExecutionPlan<async_graphql_value::Value, ConstValue>, String> {
        let mut fields = Vec::new();
        let mut variables = HashMap::new();

        for fragment in self.document.fragments.values() {
            let on_type = fragment.node.type_condition.node.on.node.as_str();
            fields.extend(self.iter(&fragment.node.selection_set.node, on_type, None));
        }

        let operation_variables = |operation: &Positioned<OperationDefinition>| {
            operation
                .node
                .variable_definitions
                .clone()
                .into_iter()
                .filter_map(move |var| {
                    let value = var.node.default_value?;
                    Some((var.node.name.node.to_string(), value.node))
                })
        };

        match &self.document.operations {
            DocumentOperations::Single(single) => {
                variables.extend(operation_variables(single));
                let name = self.get_type(single.node.ty).ok_or(format!(
                    "Root Operation type not defined for {}",
                    single.node.ty
                ))?;
                fields.extend(self.iter(&single.node.selection_set.node, name, None));
            }
            DocumentOperations::Multiple(multiple) => {
                for single in multiple.values() {
                    variables.extend(operation_variables(single));
                    let name = self.get_type(single.node.ty).ok_or(format!(
                        "Root Operation type not defined for {}",
                        single.node.ty
                    ))?;
                    fields.extend(self.iter(&single.node.selection_set.node, name, None));
                }
            }
        }

        if let Some(vars) = self.variables.as_ref() {
            variables.extend(
                vars.iter()
                    .map(|(name, value)| (name.clone(), value.clone())),
            );
        }

        Ok(ExecutionPlan::new(fields, variables))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::core::blueprint::Blueprint;
    use crate::core::config::Config;
    use crate::core::jit::builder::ConstBuilder;
    use crate::core::valid::Validator;

    const CONFIG: &str = include_str!("./fixtures/jsonplaceholder-mutation.graphql");

    fn plan(
        query: impl AsRef<str>,
    ) -> ExecutionPlan<async_graphql_value::Value, async_graphql::Value> {
        let config = Config::from_sdl(CONFIG).to_result().unwrap();
        let blueprint = Blueprint::try_from(&config.into()).unwrap();
        let document = async_graphql::parser::parse_query(query).unwrap();

        ConstBuilder::new(&blueprint, document, None)
            .build()
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
