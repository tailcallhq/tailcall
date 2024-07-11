use std::collections::HashMap;

use async_graphql::parser::types::{
    DocumentOperations, ExecutableDocument, FragmentDefinition, OperationType, Selection,
    SelectionSet,
};
use async_graphql::Positioned;
use async_graphql_value::Value;

use super::model::*;
use crate::core::blueprint::{Blueprint, Index, QueryField};
use crate::core::counter::{Count, Counter};
use crate::core::merge_right::MergeRight;

#[derive(PartialEq)]
enum Condition {
    Skip,
    Include,
    Variable(Variable),
}

struct Conditions {
    skip: Option<Condition>,
    include: Option<Condition>,
}

impl Conditions {
    // If include is always false xor skip is always true, then we can skip the
    // field
    fn is_const_skip(&self) -> bool {
        let skip = self
            .skip
            .as_ref()
            .map(|v| *v == Condition::Skip)
            .unwrap_or(false);
        let include = self
            .include
            .as_ref()
            .map(|v| *v == Condition::Skip)
            .unwrap_or(false);
        skip ^ include
    }

    fn into_variable_tuple(self) -> (Option<Variable>, Option<Variable>) {
        let include = || {
            let condition = self.include?;
            match condition {
                Condition::Variable(var) => Some(var),
                _ => None,
            }
        };

        let skip = || {
            let condition = self.skip?;
            match condition {
                Condition::Variable(var) => Some(var),
                _ => None,
            }
        };

        (include(), skip())
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
        let mut conditions = Conditions { skip: None, include: None };

        for directive in directives {
            match &*directive.node.name.node {
                "skip" => {
                    if let Some(condition_input) = directive.node.get_argument("if") {
                        let value = &condition_input.node;
                        let skip = match value {
                            Value::Variable(var) => {
                                Condition::Variable(Variable::new(var.as_str()))
                            }
                            Value::Boolean(bool) => {
                                if *bool {
                                    Condition::Skip
                                } else {
                                    Condition::Include
                                }
                            }
                            _ => Condition::Include,
                        };
                        conditions.skip = Some(skip);
                    }
                }
                "include" => {
                    if let Some(condition_input) = directive.node.get_argument("if") {
                        let value = &condition_input.node;
                        let include = match value {
                            Value::Variable(var) => {
                                Condition::Variable(Variable::new(var.as_str()))
                            }
                            Value::Boolean(bool) => {
                                if *bool {
                                    Condition::Include
                                } else {
                                    Condition::Skip
                                }
                            }
                            _ => Condition::Include,
                        };
                        conditions.include = Some(include);
                    }
                }
                _ if conditions.include.is_some() && conditions.skip.is_some() => break,
                _ => (),
            }
        }
        conditions
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
    pub fn build(&self) -> Result<ExecutionPlan, String> {
        let mut fields = Vec::new();
        let mut fragments: HashMap<&str, &FragmentDefinition> = HashMap::new();

        for (name, fragment) in self.document.fragments.iter() {
            fragments.insert(name.as_str(), &fragment.node);
        }

        match &self.document.operations {
            DocumentOperations::Single(single) => {
                let name = self.get_type(single.node.ty).ok_or(format!(
                    "Root Operation type not defined for {}",
                    single.node.ty
                ))?;
                fields.extend(self.iter(&single.node.selection_set.node, name, None, &fragments));
            }
            DocumentOperations::Multiple(multiple) => {
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

    fn plan(query: impl AsRef<str>) -> ExecutionPlan {
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
