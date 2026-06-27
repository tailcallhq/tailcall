use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display, Formatter};
use std::num::NonZeroU64;
use std::sync::Arc;

use async_graphql::parser::types::{
    ConstDirective, Directive as GqlDirective, DocumentOperations, ExecutableDocument,
    Field as GqlField, InlineFragment, OperationDefinition, OperationType, Selection, SelectionSet,
    TypeCondition,
};
use async_graphql::{Name, Positioned as AsyncPositioned, ServerError};
use async_graphql_value::{ConstValue, Value};
use serde::{Deserialize, Serialize};

use super::Error;
use crate::core::blueprint::Index;
use crate::core::ir::model::IR;
use crate::core::ir::TypedValue;
use crate::core::json::{JsonLike, JsonLikeOwned};
use crate::core::path::PathString;
use crate::core::scalar::Scalar;

#[derive(Debug, Deserialize, Clone)]
pub struct Variables<Value>(HashMap<String, Value>);

impl<V: JsonLikeOwned + Display> PathString for Variables<V> {
    fn path_string<'a, T: AsRef<str>>(&'a self, path: &'a [T]) -> Option<Cow<'a, str>> {
        self.get(path[0].as_ref())
            .map(|v| Cow::Owned(v.to_string()))
    }
}

impl<Value> Default for Variables<Value> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Value> Variables<Value> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.0.get(key)
    }
    pub fn into_hashmap(self) -> HashMap<String, Value> {
        self.0
    }

    pub fn insert(&mut self, key: String, value: Value) {
        self.0.insert(key, value);
    }
    pub fn try_map<Output, Error>(
        self,
        map: impl Fn(Value) -> Result<Output, Error>,
    ) -> Result<Variables<Output>, Error> {
        let mut hm = HashMap::new();
        for (k, v) in self.0 {
            hm.insert(k, map(v)?);
        }
        Ok(Variables(hm))
    }
}

impl<V> FromIterator<(String, V)> for Variables<V> {
    fn from_iter<T: IntoIterator<Item = (String, V)>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<Input> Field<Input> {
    #[inline(always)]
    pub fn skip<'json, Value: JsonLike<'json>>(&self, variables: &Variables<Value>) -> bool {
        let eval =
            |variable_option: Option<&Variable>, variables: &Variables<Value>, default: bool| {
                variable_option
                    .map(|a| a.as_str())
                    .and_then(|name| variables.get(name))
                    .and_then(|value| value.as_bool())
                    .unwrap_or(default)
            };
        let skip = eval(self.skip.as_ref(), variables, false);
        let include = eval(self.include.as_ref(), variables, true);

        skip == include
    }

    /// Returns the __typename of the value related to this field
    pub fn value_type<'a, Output>(&'a self, value: &'a Output) -> &'a str
    where
        Output: TypedValue<'a>,
    {
        value.get_type_name().unwrap_or(self.type_of.name())
    }

    pub fn iter(&self) -> impl Iterator<Item = &Field<Input>> {
        self.selection.iter()
    }
}

#[derive(Debug, Clone)]
pub struct Arg<Input> {
    pub id: ArgId,
    pub name: String,
    pub type_of: crate::core::Type,
    pub value: Option<Input>,
    pub default_value: Option<Input>,
}

impl<Input: Display> Display for Arg<Input> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let v = self
            .value
            .as_ref()
            .map(|v| v.to_string())
            .unwrap_or_else(|| {
                self.default_value
                    .as_ref()
                    .map(|v| v.to_string())
                    .unwrap_or_default()
            });
        write!(f, "{}: {}", self.name, v)
    }
}

impl<Input> Arg<Input> {
    pub fn try_map<Output, Error>(
        self,
        map: &impl Fn(Input) -> Result<Output, Error>,
    ) -> Result<Arg<Output>, Error> {
        Ok(Arg {
            id: self.id,
            name: self.name,
            type_of: self.type_of,
            value: self.value.map(map).transpose()?,
            default_value: self.default_value.map(map).transpose()?,
        })
    }
}

#[derive(Clone)]
pub struct ArgId(usize);

impl Debug for ArgId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ArgId {
    pub fn new(id: usize) -> Self {
        ArgId(id)
    }
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FieldId(usize);

impl Debug for FieldId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FieldId {
    pub fn new(id: usize) -> Self {
        FieldId(id)
    }
    pub fn as_usize(&self) -> usize {
        self.0
    }
}

#[derive(Clone)]
pub struct Field<Input> {
    pub id: FieldId,
    /// Name of key in the value object for this field
    pub name: String,
    /// Output name (i.e. with alias) that should be used for the result value
    /// of this field
    pub output_name: String,
    pub ir: Option<IR>,
    pub type_of: crate::core::Type,
    /// Specifies the name of type used in condition to fetch that field
    /// The type could be anything from graphql type system:
    /// interface, type, union, input type.
    /// See [spec](https://spec.graphql.org/October2021/#sec-Type-Conditions)
    pub type_condition: Option<String>,
    pub skip: Option<Variable>,
    pub include: Option<Variable>,
    pub args: Vec<Arg<Input>>,
    pub selection: Vec<Field<Input>>,
    pub parent_fragment: Option<String>,
    pub pos: Pos,
    pub directives: Vec<Directive<Input>>,
    pub is_enum: bool,
    pub scalar: Option<Scalar>,
}

pub struct DFS<'a, Input> {
    stack: Vec<std::slice::Iter<'a, Field<Input>>>,
}

impl<'a, Input> Iterator for DFS<'a, Input> {
    type Item = &'a Field<Input>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(iter) = self.stack.last_mut() {
            if let Some(field) = iter.next() {
                self.stack.push(field.selection.iter());
                return Some(field);
            } else {
                self.stack.pop();
            }
        }
        None
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Variable(String);

impl Variable {
    pub fn new(name: String) -> Self {
        Variable(name)
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
    pub fn into_string(self) -> String {
        self.0
    }
}

impl<Input> Field<Input> {
    pub fn try_map<Output, Error>(
        self,
        map: &impl Fn(Input) -> Result<Output, Error>,
    ) -> Result<Field<Output>, Error> {
        Ok(Field {
            id: self.id,
            name: self.name,
            output_name: self.output_name,
            ir: self.ir,
            type_of: self.type_of,
            type_condition: self.type_condition,
            selection: self
                .selection
                .into_iter()
                .map(|f| f.try_map(map))
                .collect::<Result<Vec<Field<Output>>, Error>>()?,
            parent_fragment: None,
            skip: self.skip,
            include: self.include,
            pos: self.pos,
            args: self
                .args
                .into_iter()
                .map(|arg| arg.try_map(map))
                .collect::<Result<_, _>>()?,
            directives: self
                .directives
                .into_iter()
                .map(|directive| directive.try_map(map))
                .collect::<Result<_, _>>()?,
            is_enum: self.is_enum,
            scalar: self.scalar,
        })
    }
}

impl<Input: Debug> Debug for Field<Input> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut debug_struct = f.debug_struct("Field");
        debug_struct.field("id", &self.id);
        debug_struct.field("name", &self.name);
        debug_struct.field("output_name", &self.output_name);
        if self.ir.is_some() {
            debug_struct.field("ir", &"Some(..)");
        }
        debug_struct.field("type_of", &self.type_of);
        debug_struct.field("type_condition", &self.type_condition);
        if !self.args.is_empty() {
            debug_struct.field("args", &self.args);
        }
        if !self.selection.is_empty() {
            debug_struct.field("selection", &self.selection);
        }
        if self.skip.is_some() {
            debug_struct.field("skip", &self.skip);
        }
        if self.include.is_some() {
            debug_struct.field("include", &self.include);
        }
        debug_struct.field("directives", &self.directives);

        debug_struct.finish()
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct OPHash(u64);

impl OPHash {
    pub fn new(hash: u64) -> Self {
        OPHash(hash)
    }
}

#[derive(Debug, Clone)]
pub struct OperationPlan<Input> {
    pub root_name: String,
    pub operation_type: OperationType,
    // TODO: drop index from here. Embed all the necessary information in each field of the plan.
    pub index: Arc<Index>,
    pub is_introspection_query: bool,
    pub is_dedupe: bool,
    pub is_const: bool,
    pub is_protected: bool,
    pub min_cache_ttl: Option<NonZeroU64>,
    pub selection: Vec<Field<Input>>,
    pub before: Option<IR>,
    pub interfaces: Option<HashSet<String>>,
}

impl<Input> OperationPlan<Input> {
    pub fn try_map<Output, Error>(
        self,
        map: impl Fn(Input) -> Result<Output, Error>,
    ) -> Result<OperationPlan<Output>, Error> {
        let mut selection = vec![];

        for n in self.selection {
            selection.push(n.try_map(&map)?);
        }

        Ok(OperationPlan {
            selection,
            root_name: self.root_name,
            operation_type: self.operation_type,
            index: self.index,
            is_introspection_query: self.is_introspection_query,
            is_dedupe: self.is_dedupe,
            is_const: self.is_const,
            is_protected: self.is_protected,
            min_cache_ttl: self.min_cache_ttl,
            before: self.before,
            interfaces: None,
        })
    }
}

impl<Input> OperationPlan<Input> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        root_name: &str,
        selection: Vec<Field<Input>>,
        operation_type: OperationType,
        index: Arc<Index>,
        is_introspection_query: bool,
        interfaces: Option<HashSet<String>>,
    ) -> Self
    where
        Input: Clone,
    {
        Self {
            root_name: root_name.to_string(),
            selection,
            operation_type,
            index,
            is_introspection_query,
            is_dedupe: false,
            is_const: false,
            is_protected: false,
            min_cache_ttl: None,
            before: Default::default(),
            interfaces,
        }
    }

    /// Returns the name of the root type
    pub fn root_name(&self) -> &str {
        &self.root_name
    }

    /// Returns a graphQL operation type
    pub fn operation_type(&self) -> OperationType {
        self.operation_type
    }

    /// Check if current graphQL operation is query
    pub fn is_query(&self) -> bool {
        self.operation_type == OperationType::Query
    }

    /// Returns a flat [Field] representation
    pub fn iter_dfs(&self) -> DFS<'_, Input> {
        DFS { stack: vec![self.selection.iter()] }
    }

    /// Returns number of fields in plan
    pub fn size(&self) -> usize {
        fn count<A>(field: &Field<A>) -> usize {
            1 + field.selection.iter().map(count).sum::<usize>()
        }
        self.selection.iter().map(count).sum()
    }

    /// Check if the field is of scalar type
    pub fn field_is_scalar(&self, field: &Field<Input>) -> bool {
        self.index.type_is_scalar(field.type_of.name())
    }

    /// Check if the field is of enum type
    pub fn field_is_enum(&self, field: &Field<Input>) -> bool {
        self.index.type_is_enum(field.type_of.name())
    }

    /// Validate the value against enum variants of the field
    pub fn field_validate_enum_value(&self, field: &Field<Input>, value: &str) -> bool {
        self.index.validate_enum_value(field.type_of.name(), value)
    }

    pub fn field_is_part_of_value<'a, Output>(
        &'a self,
        field: &'a Field<Input>,
        value: &'a Output,
    ) -> bool
    where
        Output: TypedValue<'a>,
    {
        match &field.type_condition {
            Some(type_condition) => match value.get_type_name() {
                Some(value_type) => self.index.is_type_implements(value_type, type_condition),
                // if there is no __typename in value that means there is a bug in implementation
                // such we haven't resolved the concrete type or type shouldn't be
                // inferred here at all and we should just use the field
                None => true,
            },
            // if there is no type_condition restriction then use this field
            None => true,
        }
    }
    /// returns true if plan is dedupable
    pub fn can_dedupe(&self) -> bool {
        self.is_query() && (self.is_dedupe || self.is_const || self.min_cache_ttl.is_some())
    }
}

impl<Input> OperationPlan<Input>
where
    Input: Clone + Into<Value>,
{
    /// Builds an executable GraphQL document from this operation plan.
    pub fn to_doc(&self) -> ExecutableDocument {
        ExecutableDocument {
            operations: DocumentOperations::Single(AsyncPositioned::new(
                OperationDefinition {
                    ty: self.operation_type,
                    variable_definitions: vec![],
                    directives: vec![],
                    selection_set: self.selection_set_to_doc(self.selection.iter(), true),
                },
                Default::default(),
            )),
            fragments: HashMap::new(),
        }
    }

    fn selection_set_to_doc<'a>(
        &self,
        fields: impl IntoIterator<Item = &'a Field<Input>>,
        wrap_fragments: bool,
    ) -> AsyncPositioned<SelectionSet>
    where
        Input: 'a,
    {
        let fields = fields.into_iter().collect::<Vec<_>>();
        let pos = fields
            .first()
            .map(|field| field.pos.into())
            .unwrap_or_default();
        let mut items = Vec::new();
        let mut fragment_groups: Vec<(&str, Vec<&Field<Input>>)> = Vec::new();

        for field in fields {
            if wrap_fragments {
                if let Some(fragment) = field.parent_fragment.as_deref() {
                    if let Some((_, fields)) = fragment_groups
                        .iter_mut()
                        .find(|(name, _)| *name == fragment)
                    {
                        fields.push(field);
                    } else {
                        fragment_groups.push((fragment, vec![field]));
                    }
                    continue;
                }
            }

            items.push(self.field_selection_to_doc(field));
        }

        for (fragment, fields) in fragment_groups {
            let pos = fields
                .first()
                .map(|field| field.pos.into())
                .unwrap_or_default();

            items.push(AsyncPositioned::new(
                Selection::InlineFragment(AsyncPositioned::new(
                    InlineFragment {
                        type_condition: Some(AsyncPositioned::new(
                            TypeCondition { on: AsyncPositioned::new(Name::new(fragment), pos) },
                            pos,
                        )),
                        directives: vec![],
                        selection_set: self.selection_set_to_doc(fields, false),
                    },
                    pos,
                )),
                pos,
            ));
        }

        AsyncPositioned::new(SelectionSet { items }, pos)
    }

    fn field_selection_to_doc(&self, field: &Field<Input>) -> AsyncPositioned<Selection> {
        let pos = field.pos.into();
        AsyncPositioned::new(
            Selection::Field(AsyncPositioned::new(self.field_to_doc(field), pos)),
            pos,
        )
    }

    fn field_to_doc(&self, field: &Field<Input>) -> GqlField {
        let pos = field.pos.into();
        GqlField {
            alias: (field.output_name != field.name)
                .then(|| AsyncPositioned::new(Name::new(&field.output_name), pos)),
            name: AsyncPositioned::new(Name::new(&field.name), pos),
            arguments: field
                .args
                .iter()
                .filter_map(|arg| {
                    arg.value.as_ref().map(|value| {
                        (
                            AsyncPositioned::new(Name::new(&arg.name), pos),
                            AsyncPositioned::new(value.clone().into(), pos),
                        )
                    })
                })
                .collect(),
            directives: self.field_directives_to_doc(field),
            selection_set: self.selection_set_to_doc(field.selection.iter(), true),
        }
    }

    fn field_directives_to_doc(&self, field: &Field<Input>) -> Vec<AsyncPositioned<GqlDirective>> {
        let pos = field.pos.into();
        let mut directives = field
            .directives
            .iter()
            .map(|directive| {
                AsyncPositioned::new(
                    GqlDirective {
                        name: AsyncPositioned::new(Name::new(&directive.name), pos),
                        arguments: directive
                            .arguments
                            .iter()
                            .map(|(name, value)| {
                                (
                                    AsyncPositioned::new(Name::new(name), pos),
                                    AsyncPositioned::new(value.clone().into(), pos),
                                )
                            })
                            .collect(),
                    },
                    pos,
                )
            })
            .collect::<Vec<_>>();

        if let Some(skip) = &field.skip {
            directives.push(Self::condition_directive_to_doc("skip", skip.as_str(), pos));
        }

        if let Some(include) = &field.include {
            directives.push(Self::condition_directive_to_doc(
                "include",
                include.as_str(),
                pos,
            ));
        }

        directives
    }

    fn condition_directive_to_doc(
        name: &str,
        variable: &str,
        pos: async_graphql::Pos,
    ) -> AsyncPositioned<GqlDirective> {
        AsyncPositioned::new(
            GqlDirective {
                name: AsyncPositioned::new(Name::new(name), pos),
                arguments: vec![(
                    AsyncPositioned::new(Name::new("if"), pos),
                    AsyncPositioned::new(Value::Variable(Name::new(variable)), pos),
                )],
            },
            pos,
        )
    }
}

#[derive(Clone, Debug)]
pub struct Directive<Input> {
    pub name: String,
    pub arguments: Vec<(String, Input)>,
}

impl<Input> Directive<Input> {
    pub fn try_map<Output, Error>(
        self,
        map: &impl Fn(Input) -> Result<Output, Error>,
    ) -> Result<Directive<Output>, Error> {
        Ok(Directive {
            name: self.name,
            arguments: self
                .arguments
                .into_iter()
                .map(|(k, v)| map(v).map(|mapped_value| (k, mapped_value)))
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl<'a> From<&'a Directive<ConstValue>> for ConstDirective {
    fn from(value: &'a Directive<ConstValue>) -> Self {
        // we don't use pos required in Positioned struct, hence using defaults.
        ConstDirective {
            name: AsyncPositioned::new(Name::new(&value.name), Default::default()),
            arguments: value
                .arguments
                .iter()
                .map(|a| {
                    (
                        AsyncPositioned::new(Name::new(a.0.clone()), Default::default()),
                        AsyncPositioned::new(a.1.clone(), Default::default()),
                    )
                })
                .collect::<Vec<_>>(),
        }
    }
}

/// Original position of an element in source code.
///
/// You can serialize and deserialize it to the GraphQL `locations` format
/// ([reference](https://spec.graphql.org/October2021/#sec-Errors)).
#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Copy, Default, Hash, Serialize, Deserialize)]
pub struct Pos {
    /// One-based line number.
    pub line: usize,

    /// One-based column number.
    pub column: usize,
}

impl std::fmt::Debug for Pos {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Pos({}:{})", self.line, self.column)
    }
}

impl std::fmt::Display for Pos {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

impl From<async_graphql::Pos> for Pos {
    fn from(pos: async_graphql::Pos) -> Self {
        Self { line: pos.line, column: pos.column }
    }
}

impl From<Pos> for async_graphql::Pos {
    fn from(value: Pos) -> Self {
        async_graphql::Pos { line: value.line, column: value.column }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PathSegment<'a> {
    /// A field in an object.
    Field(Cow<'a, String>),
    /// An index in a list.
    Index(usize),
}

impl From<async_graphql::PathSegment> for PathSegment<'static> {
    fn from(value: async_graphql::PathSegment) -> Self {
        match value {
            async_graphql::PathSegment::Field(field) => PathSegment::Field(Cow::Owned(field)),
            async_graphql::PathSegment::Index(index) => PathSegment::Index(index),
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct Positioned<Value> {
    pub value: Value,
    pub pos: Pos,
    pub path: Vec<PathSegment<'static>>,
}

impl<Value> Positioned<Value> {
    pub fn new(value: Value, pos: Pos) -> Self {
        Positioned { value, pos, path: vec![] }
    }
}

impl<Value> Positioned<Value>
where
    Value: Clone,
{
    pub fn with_path(&mut self, path: Vec<PathSegment<'static>>) -> Self {
        Self { value: self.value.clone(), pos: self.pos, path }
    }
}

// TODO: Improve conversion logic to avoid unnecessary round-trip conversions
//       between ServerError and Positioned<Error>.
impl From<ServerError> for Positioned<Error> {
    fn from(val: ServerError) -> Self {
        Self {
            value: Error::ServerError(val.clone()),
            pos: val.locations.first().cloned().unwrap_or_default().into(),
            path: val
                .path
                .into_iter()
                .map(PathSegment::from)
                .collect::<Vec<_>>(),
        }
    }
}

#[cfg(test)]
mod test {
    use async_graphql::parser::types::{ConstDirective, DocumentOperations, Selection};
    use async_graphql::Request;
    use async_graphql_value::{ConstValue, Value};

    use super::{Directive, OperationPlan};
    use crate::core::blueprint::Blueprint;
    use crate::core::config::ConfigModule;
    use crate::core::jit;
    use crate::include_config;

    fn plan(query: &str) -> OperationPlan<async_graphql_value::Value> {
        let config = include_config!("./fixtures/dedupe.graphql").unwrap();
        let module = ConfigModule::from(config);
        let bp = Blueprint::try_from(&module).unwrap();

        let request = Request::new(query);
        let jit_request = jit::Request::from(request);
        jit_request.create_plan(&bp).unwrap()
    }

    #[test]
    fn test_from_custom_directive() {
        let custom_directive = Directive {
            name: "options".to_string(),
            arguments: vec![("paging".to_string(), ConstValue::Boolean(true))],
        };

        let async_directive: ConstDirective = (&custom_directive).into();
        insta::assert_debug_snapshot!(async_directive);
    }

    #[test]
    fn test_operation_plan_dedupe() {
        let actual = plan(r#"{ posts { id } }"#);

        assert!(!actual.is_dedupe);
    }

    #[test]
    fn test_operation_plan_dedupe_nested() {
        let actual = plan(r#"{ posts { id users { id } } }"#);

        assert!(!actual.is_dedupe);
    }

    #[test]
    fn test_operation_plan_dedupe_false() {
        let actual = plan(r#"{ users { id comments {body} } }"#);

        assert!(actual.is_dedupe);
    }

    #[test]
    fn test_operation_plan_to_doc_preserves_selection_fields() {
        let actual = plan(
            r#"
            query($includeName: Boolean! = true) {
                users {
                    id @options(paging: $includeName)
                    displayName: name @include(if: $includeName)
                }
            }
            "#,
        );

        let doc = actual.to_doc();
        let operation = match &doc.operations {
            DocumentOperations::Single(operation) => &operation.node,
            DocumentOperations::Multiple(_) => panic!("expected single operation"),
        };

        assert!(doc.fragments.is_empty());
        assert!(operation.variable_definitions.is_empty());
        assert_eq!(
            operation.ty,
            async_graphql::parser::types::OperationType::Query
        );

        let users = match &operation.selection_set.node.items[0].node {
            Selection::Field(field) => &field.node,
            _ => panic!("expected users field"),
        };
        assert_eq!(users.name.node.as_str(), "users");
        assert!(users.alias.is_none());

        let id = match &users.selection_set.node.items[0].node {
            Selection::Field(field) => &field.node,
            _ => panic!("expected id field"),
        };
        assert_eq!(id.name.node.as_str(), "id");
        assert_eq!(id.directives.len(), 1);
        assert_eq!(id.directives[0].node.name.node.as_str(), "options");
        assert_eq!(id.directives[0].node.arguments[0].0.node.as_str(), "paging");
        assert!(matches!(
            &id.directives[0].node.arguments[0].1.node,
            Value::Variable(name) if name.as_str() == "includeName"
        ));

        let name = match &users.selection_set.node.items[1].node {
            Selection::Field(field) => &field.node,
            _ => panic!("expected name field"),
        };
        assert_eq!(name.name.node.as_str(), "name");
        assert_eq!(name.alias.as_ref().unwrap().node.as_str(), "displayName");
        assert_eq!(name.directives.len(), 1);
        assert_eq!(name.directives[0].node.name.node.as_str(), "include");
        assert_eq!(name.directives[0].node.arguments[0].0.node.as_str(), "if");
        assert!(matches!(
            &name.directives[0].node.arguments[0].1.node,
            Value::Variable(include) if include.as_str() == "includeName"
        ));
    }
}
