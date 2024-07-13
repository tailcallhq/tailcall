use std::collections::HashMap;
use std::ops::Deref;

use async_graphql::parser::types::{
    Directive, DocumentOperations, ExecutableDocument, FragmentDefinition, OperationType,
    Selection, SelectionSet,
};
use async_graphql::Positioned;
use async_graphql_value::Value;

use super::model::*;
use crate::core::blueprint::{Blueprint, Index, QueryField};
use crate::core::counter::{Count, Counter};
use crate::core::merge_right::MergeRight;

#[derive(PartialEq)]
enum Condition {
    True,
    False,
    Variable(Variable),
}

struct Conditions {
    skip: Option<Condition>,
    include: Option<Condition>,
}

impl Conditions {
    /// Checks if the field should be skipped always
    fn is_const_skip(&self) -> bool {
        matches!(self.skip, Some(Condition::True)) ^ matches!(self.include, Some(Condition::True))
    }

    fn into_variable_tuple(self) -> (Option<Variable>, Option<Variable>) {
        let comp = |condition| match condition? {
            Condition::Variable(var) => Some(var),
            _ => None,
        };

        let include = comp(self.include);
        let skip = comp(self.skip);

        (include, skip)
    }
}

pub struct Builder {
    pub index: Index,
    pub arg_id: Counter<usize>,
    pub field_id: Counter<usize>,
    pub document: ExecutableDocument,
}

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

    #[inline(always)]
    fn include(
        &self,
        directives: &[Positioned<async_graphql::parser::types::Directive>],
    ) -> Conditions {
        fn get_condition(dir: &Directive) -> Option<Condition> {
            let arg = dir.get_argument("if").map(|pos| &pos.node);
            let is_include = dir.name.node.as_str() == "include";
            match arg {
                None => None,
                Some(value) => match value {
                    Value::Boolean(bool) => {
                        let condition = if is_include ^ bool {
                            Condition::True
                        } else {
                            Condition::False
                        };
                        Some(condition)
                    }
                    Value::Variable(var) => {
                        Some(Condition::Variable(Variable::new(var.deref().to_owned())))
                    }
                    _ => None,
                },
            }
        }
        Conditions {
            skip: directives
                .iter()
                .find(|d| d.node.name.node.as_str() == "skip")
                .map(|d| &d.node)
                .and_then(get_condition),
            include: directives
                .iter()
                .find(|d| d.node.name.node.as_str() == "include")
                .map(|d| &d.node)
                .and_then(get_condition),
        }
    }

    #[allow(clippy::too_many_arguments)]
    #[inline(always)]
    fn iter(
        &self,
        selection: &SelectionSet,
        type_of: &str,
        exts: Option<Flat>,
        fragments: &HashMap<&str, &FragmentDefinition>,
    ) -> Vec<Field<Flat>> {
        let mut fields = vec![];
        for selection in &selection.items {
            match &selection.node {
                Selection::Field(Positioned { node: gql_field, .. }) => {
                    let conditions = self.include(&gql_field.directives);

                    // if include is always false xor skip is always true,
                    // then we can skip the field from the plan
                    if conditions.is_const_skip() {
                        continue;
                    }

                    let (include, skip) = conditions.into_variable_tuple();

                    let field_name = gql_field.name.node.as_str();
                    let field_args = gql_field
                        .arguments
                        .iter()
                        .map(|(k, v)| (k.node.as_str().to_string(), v.node.to_owned()))
                        .collect::<HashMap<_, _>>();

                    if let Some(field_def) = self.index.get_field(type_of, field_name) {
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
                                args.push(Arg {
                                    id,
                                    name,
                                    type_of,
                                    value: Some(value),
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
                            skip,
                            include,
                            args,
                            extensions: exts.clone(),
                        });
                        fields = fields.merge_right(child_fields);
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

    #[inline(always)]
    fn get_type(&self, ty: OperationType) -> Option<&str> {
        match ty {
            OperationType::Query => Some(self.index.get_query()),
            OperationType::Mutation => self.index.get_mutation(),
            OperationType::Subscription => None,
        }
    }

    #[inline(always)]
    pub fn build(&self) -> Result<OperationPlan, String> {
        let mut fields = Vec::new();
        let mut fragments: HashMap<&str, &FragmentDefinition> = HashMap::new();

        for (name, fragment) in self.document.fragments.iter() {
            fragments.insert(name.as_str(), &fragment.node);
        }

        let operation_type = match &self.document.operations {
            DocumentOperations::Single(single) => {
                let name = self.get_type(single.node.ty).ok_or(format!(
                    "Root Operation type not defined for {}",
                    single.node.ty
                ))?;
                fields.extend(self.iter(&single.node.selection_set.node, name, None, &fragments));
                single.node.ty
            }
            DocumentOperations::Multiple(multiple) => {
                let mut operation_type = OperationType::Query;
                for single in multiple.values() {
                    let name = self.get_type(single.node.ty).ok_or(format!(
                        "Root Operation type not defined for {}",
                        single.node.ty
                    ))?;
                    fields.extend(self.iter(
                        &single.node.selection_set.node,
                        name,
                        None,
                        &fragments,
                    ));
                    operation_type = single.node.ty;
                }
                operation_type
            }
        };

        Ok(OperationPlan::new(fields, operation_type))
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

    fn plan(query: impl AsRef<str>) -> OperationPlan {
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
