use std::collections::HashMap;
use std::ops::Deref;

use async_graphql::parser::types::{
    Directive, DocumentOperations, ExecutableDocument, FragmentDefinition, OperationDefinition,
    OperationType, Selection, SelectionSet,
};
use async_graphql::Positioned;
use async_graphql_value::{ConstValue, Value};

use super::input_resolver::InputResolver;
use super::model::*;
use super::BuildError;
use crate::core::blueprint::{Blueprint, Index, QueryField};
use crate::core::counter::{Count, Counter};
use crate::core::jit::model::OperationPlan;
use crate::core::merge_right::MergeRight;

#[derive(PartialEq, strum_macros::Display)]
enum Condition {
    True,
    False,
    Variable(Variable),
}

struct Conditions {
    skip: Condition,
    include: Condition,
}

impl Conditions {
    /// Checks if the field should be skipped always
    fn is_const_skip(&self) -> bool {
        // Truth Table
        // skip | include | ignore
        // T   |    T    |   T
        // T   |    F    |   T
        // F   |    T    |   F
        // F   |    F    |   T
        //
        // Logical expression:
        //     say skip = p, include = q
        // (p V ~q)

        // so instead of a normalizing variables,
        // we can just check for the above condition

        matches!(
            (&self.skip, &self.include),
            (Condition::True, _) | (Condition::False, Condition::False)
        )
    }

    fn into_variable_tuple(self) -> (Option<Variable>, Option<Variable>) {
        let comp = |condition| match condition {
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

    #[inline(always)]
    fn include(
        &self,
        directives: &[Positioned<async_graphql::parser::types::Directive>],
    ) -> Conditions {
        fn get_condition(dir: &Directive) -> Option<Condition> {
            let arg = dir.get_argument("if").map(|pos| &pos.node);
            match arg {
                None => None,
                Some(value) => match value {
                    Value::Boolean(bool) => {
                        let condition = if *bool {
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
                .and_then(get_condition)
                .unwrap_or(Condition::False),
            include: directives
                .iter()
                .find(|d| d.node.name.node.as_str() == "include")
                .map(|d| &d.node)
                .and_then(get_condition)
                .unwrap_or(Condition::True),
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
    ) -> Vec<Field<Flat, Value>> {
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
                        let name = gql_field
                            .alias
                            .as_ref()
                            .map(|alias| alias.node.to_string())
                            .unwrap_or(field_name.to_string());
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
                            pos: selection.pos,
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

    #[inline(always)]
    fn get_type(&self, ty: OperationType) -> Option<&str> {
        match ty {
            OperationType::Query => Some(self.index.get_query()),
            OperationType::Mutation => self.index.get_mutation(),
            OperationType::Subscription => None,
        }
    }

    /// Resolves currently processed operation
    /// based on [spec](https://spec.graphql.org/October2021/#sec-Executing-Requests)
    #[inline(always)]
    fn get_operation(
        &self,
        operation_name: Option<&str>,
    ) -> Result<&OperationDefinition, BuildError> {
        if let Some(operation_name) = operation_name {
            match &self.document.operations {
                DocumentOperations::Single(_) => None,
                DocumentOperations::Multiple(operations) => {
                    operations.get(operation_name).map(|op| &op.node)
                }
            }
            .ok_or_else(|| BuildError::OperationNotFound(operation_name.to_string()))
        } else {
            match &self.document.operations {
                DocumentOperations::Single(operation) => Ok(&operation.node),
                DocumentOperations::Multiple(map) if map.len() == 1 => {
                    let (_, operation) = map.iter().next().unwrap();
                    Ok(&operation.node)
                }
                DocumentOperations::Multiple(_) => Err(BuildError::OperationNameRequired),
            }
        }
    }

    #[inline(always)]
    pub fn build(
        &self,
        variables: &Variables<ConstValue>,
        operation_name: Option<&str>,
    ) -> Result<OperationPlan<ConstValue>, BuildError> {
        let mut fields = Vec::new();
        let mut fragments: HashMap<&str, &FragmentDefinition> = HashMap::new();

        for (name, fragment) in self.document.fragments.iter() {
            fragments.insert(name.as_str(), &fragment.node);
        }

        let operation = self.get_operation(operation_name)?;

        let name = self
            .get_type(operation.ty)
            .ok_or(BuildError::RootOperationTypeNotDefined { operation: operation.ty })?;
        fields.extend(self.iter(&operation.selection_set.node, name, None, &fragments));

        let plan = OperationPlan::new(fields, operation.ty);
        // TODO: operation from [ExecutableDocument] could contain definitions for
        // default values of arguments. That info should be passed to
        // [InputResolver] to resolve defaults properly
        let input_resolver = InputResolver::new(plan);

        Ok(input_resolver.resolve_input(variables)?)
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

    fn plan(
        query: impl AsRef<str>,
        variables: &Variables<ConstValue>,
    ) -> OperationPlan<ConstValue> {
        let config = Config::from_sdl(CONFIG).to_result().unwrap();
        let blueprint = Blueprint::try_from(&config.into()).unwrap();
        let document = async_graphql::parser::parse_query(query).unwrap();
        Builder::new(&blueprint, document)
            .build(variables, None)
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
            &Variables::new(),
        );
        assert!(plan.is_query());
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
            &Variables::new(),
        );

        assert!(plan.is_query());
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
            &Variables::new(),
        );

        assert!(plan.is_query());
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
            &Variables::new(),
        );

        assert!(!plan.is_query());
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
            &Variables::new(),
        );

        assert!(plan.is_query());
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
            &Variables::new(),
        );

        assert!(plan.is_query());
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
            &Variables::from_iter([("id".into(), ConstValue::from(1))]),
        );

        assert!(plan.is_query());
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
            &Variables::new(),
        );

        assert!(plan.is_query());
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
            &Variables::new(),
        );

        assert!(!plan.is_query());
        insta::assert_debug_snapshot!(plan.into_nested());
    }

    #[test]
    fn test_condition() {
        // cases:
        // skip | include | ignore
        // T  |    T    |   T
        // T  |    F    |   T
        // T  |    V    |   T
        // F  |    F    |   T
        // F  |    T    |   F
        // F  |    V    |   F
        // V  |    T    |   F
        // V  |    F    |   F

        let test_var = Variable::new("ssdd.dev".to_string());

        let test_cases = vec![
            // ignore
            (Condition::True, Condition::True, true),
            (Condition::True, Condition::False, true),
            (Condition::True, Condition::Variable(test_var.clone()), true),
            (Condition::False, Condition::False, true),
            // don't ignore
            (Condition::False, Condition::True, false),
            (
                Condition::False,
                Condition::Variable(test_var.clone()),
                false,
            ),
            (
                Condition::Variable(test_var.clone()),
                Condition::True,
                false,
            ),
            (Condition::Variable(test_var), Condition::False, false),
        ];

        for (skip, include, expected) in test_cases {
            let conditions = Conditions { skip, include };
            assert_eq!(
                conditions.is_const_skip(),
                expected,
                "Failed for skip: {}, include: {}",
                conditions.skip,
                conditions.include
            );
        }
    }

    #[test]
    fn test_resolving_operation() {
        let query = r#"
            query GetPosts {
                posts {
                    id
                    userId
                    title
                }
            }

            mutation CreateNewPost {
                createPost(post: {
                    userId: 1,
                    title: "test-12",
                    body: "test-12",
                }) {
                    id
                    userId
                    title
                    body
                }
            }
        "#;
        let config = Config::from_sdl(CONFIG).to_result().unwrap();
        let blueprint = Blueprint::try_from(&config.into()).unwrap();
        let document = async_graphql::parser::parse_query(query).unwrap();
        let error = Builder::new(&blueprint, document.clone())
            .build(&Variables::new(), None)
            .unwrap_err();

        assert_eq!(error, BuildError::OperationNameRequired);

        let error = Builder::new(&blueprint, document.clone())
            .build(&Variables::new(), Some("unknown"))
            .unwrap_err();

        assert_eq!(error, BuildError::OperationNotFound("unknown".to_string()));

        let plan = Builder::new(&blueprint, document.clone())
            .build(&Variables::new(), Some("GetPosts"))
            .unwrap();
        assert!(plan.is_query());
        insta::assert_debug_snapshot!(plan.into_nested());

        let plan = Builder::new(&blueprint, document.clone())
            .build(&Variables::new(), Some("CreateNewPost"))
            .unwrap();
        assert!(!plan.is_query());
        insta::assert_debug_snapshot!(plan.into_nested());
    }
}
