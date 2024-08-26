use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;

use async_graphql::parser::types::{ConstDirective, OperationType};
use async_graphql::{ErrorExtensions, Name, Positioned as AsyncPositioned, ServerError};
use async_graphql_value::ConstValue;
use serde::{Deserialize, Serialize};

use super::Error;
use crate::core::blueprint::Index;
use crate::core::ir::model::IR;
use crate::core::ir::TypedValue;

#[derive(Debug, Deserialize, Clone)]
pub struct Variables<Value>(HashMap<String, Value>);

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

#[derive(Debug, Clone)]
pub struct Arg<Input> {
    pub id: ArgId,
    pub name: String,
    pub type_of: crate::core::blueprint::Type,
    pub value: Option<Input>,
    pub default_value: Option<Input>,
}

impl<Input> Arg<Input> {
    pub fn try_map<Output, Error>(
        self,
        map: impl Fn(Input) -> Result<Output, Error>,
    ) -> Result<Arg<Output>, Error> {
        Ok(Arg {
            id: self.id,
            name: self.name,
            type_of: self.type_of,
            value: self.value.map(&map).transpose()?,
            default_value: self.default_value.map(&map).transpose()?,
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

#[derive(Clone, PartialEq, Eq, Hash)]
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
pub struct Field<Extensions, Input> {
    pub id: FieldId,
    /// Name of key in the value object for this field
    pub name: String,
    /// Output name (i.e. with alias) that should be used for the result value
    /// of this field
    pub output_name: String,
    pub ir: Option<IR>,
    pub type_of: crate::core::blueprint::Type,
    /// Specifies the name of type used in condition to fetch that field
    /// The type could be anything from graphql type system:
    /// interface, type, union, input type.
    /// See [spec](https://spec.graphql.org/October2021/#sec-Type-Conditions)
    pub type_condition: String,
    pub skip: Option<Variable>,
    pub include: Option<Variable>,
    pub args: Vec<Arg<Input>>,
    pub extensions: Option<Extensions>,
    pub pos: Pos,
    pub directives: Vec<Directive<Input>>,
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

impl<Extensions, Input> Field<Extensions, Input> {
    pub fn value_type<'a, Output>(&'a self, value: &'a Output) -> &'a str
    where
        Output: TypedValue<'a>,
    {
        value.get_type_name().unwrap_or(self.type_of.name())
    }
}

impl<Input> Field<Nested<Input>, Input> {
    pub fn try_map<Output, Error>(
        self,
        map: &impl Fn(Input) -> Result<Output, Error>,
    ) -> Result<Field<Nested<Output>, Output>, Error> {
        let mut extensions = None;

        if let Some(nested) = self.extensions {
            let nested = nested
                .0
                .into_iter()
                .map(|v| v.try_map(map))
                .collect::<Result<_, _>>()?;
            extensions = Some(Nested(nested));
        }

        Ok(Field {
            id: self.id,
            name: self.name,
            output_name: self.output_name,
            ir: self.ir,
            type_of: self.type_of,
            type_condition: self.type_condition,
            extensions,
            pos: self.pos,
            skip: self.skip,
            include: self.include,
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
        })
    }
}

impl<Input> Field<Flat, Input> {
    pub fn try_map<Output, Error>(
        self,
        map: impl Fn(Input) -> Result<Output, Error>,
    ) -> Result<Field<Flat, Output>, Error> {
        Ok(Field {
            id: self.id,
            name: self.name,
            output_name: self.output_name,
            ir: self.ir,
            type_of: self.type_of,
            type_condition: self.type_condition,
            extensions: self.extensions,
            skip: self.skip,
            include: self.include,
            pos: self.pos,
            args: self
                .args
                .into_iter()
                .map(|arg| arg.try_map(&map))
                .collect::<Result<_, _>>()?,
            directives: self
                .directives
                .into_iter()
                .map(|directive| directive.try_map(&map))
                .collect::<Result<_, _>>()?,
        })
    }
}

impl<Input> Field<Nested<Input>, Input> {
    /// iters over children fields that satisfies
    /// passed filter_fn
    pub fn nested_iter<'a>(
        &'a self,
        mut filter_fn: impl FnMut(&'a Field<Nested<Input>, Input>) -> bool + 'a,
    ) -> impl Iterator<Item = &Field<Nested<Input>, Input>> + 'a {
        self.extensions
            .as_ref()
            .map(move |nested| nested.0.iter().filter(move |&field| filter_fn(field)))
            .into_iter()
            .flatten()
    }
}

impl<Input> Field<Flat, Input> {
    pub fn parent(&self) -> Option<&FieldId> {
        self.extensions.as_ref().map(|flat| &flat.0)
    }

    fn into_nested(self, fields: &[Field<Flat, Input>]) -> Field<Nested<Input>, Input>
    where
        Input: Clone,
    {
        let mut children = Vec::new();
        for field in fields.iter() {
            if let Some(id) = field.parent() {
                if *id == self.id {
                    children.push(field.to_owned().into_nested(fields));
                }
            }
        }

        let extensions = if children.is_empty() {
            None
        } else {
            Some(Nested(children))
        };

        Field {
            id: self.id,
            name: self.name,
            output_name: self.output_name,
            ir: self.ir,
            type_of: self.type_of,
            type_condition: self.type_condition,
            skip: self.skip,
            include: self.include,
            args: self.args,
            pos: self.pos,
            extensions,
            directives: self.directives,
        }
    }
}

impl<Extensions: Debug, Input: Debug> Debug for Field<Extensions, Input> {
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
        if self.extensions.is_some() {
            debug_struct.field("extensions", &self.extensions);
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

/// Stores field relationships in a flat structure where each field links to its
/// parent.
#[derive(Clone, Debug)]
pub struct Flat(FieldId);

impl Flat {
    pub fn new(parent_id: FieldId) -> Self {
        Flat(parent_id)
    }
}

/// Store field relationships in a nested structure like a tree where each field
/// links to its children.
#[derive(Clone, Debug)]
pub struct Nested<Input>(Vec<Field<Nested<Input>, Input>>);

#[derive(Clone)]
pub struct OperationPlan<Input> {
    flat: Vec<Field<Flat, Input>>,
    operation_type: OperationType,
    nested: Vec<Field<Nested<Input>, Input>>,
    // TODO: drop index from here. Embed all the necessary information in each field of the plan.
    pub index: Arc<Index>,
}

impl<Input> std::fmt::Debug for OperationPlan<Input> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OperationPlan")
            .field("operation_type", &self.operation_type)
            .finish()
    }
}

impl<Input> OperationPlan<Input> {
    pub fn try_map<Output, Error>(
        self,
        map: impl Fn(Input) -> Result<Output, Error>,
    ) -> Result<OperationPlan<Output>, Error> {
        let mut flat = vec![];

        for f in self.flat {
            flat.push(f.try_map(&map)?);
        }

        let mut nested = vec![];

        for n in self.nested {
            nested.push(n.try_map(&map)?);
        }

        Ok(OperationPlan {
            flat,
            operation_type: self.operation_type,
            nested,
            index: self.index,
        })
    }
}

impl<Input> OperationPlan<Input> {
    pub fn new(
        fields: Vec<Field<Flat, Input>>,
        operation_type: OperationType,
        index: Arc<Index>,
    ) -> Self
    where
        Input: Clone,
    {
        let nested = fields
            .clone()
            .into_iter()
            .filter(|f| f.extensions.is_none())
            .map(|f| f.into_nested(&fields))
            .collect::<Vec<_>>();

        Self { flat: fields, nested, operation_type, index }
    }

    pub fn operation_type(&self) -> OperationType {
        self.operation_type
    }

    pub fn is_query(&self) -> bool {
        self.operation_type == OperationType::Query
    }

    pub fn as_nested(&self) -> &[Field<Nested<Input>, Input>] {
        &self.nested
    }

    pub fn into_nested(self) -> Vec<Field<Nested<Input>, Input>> {
        self.nested
    }

    pub fn as_parent(&self) -> &[Field<Flat, Input>] {
        &self.flat
    }

    pub fn find_field(&self, id: FieldId) -> Option<&Field<Flat, Input>> {
        self.flat.iter().find(|field| field.id == id)
    }

    pub fn find_field_path<S: AsRef<str>>(&self, path: &[S]) -> Option<&Field<Flat, Input>> {
        match path.split_first() {
            None => None,
            Some((name, path)) => {
                let field = self.flat.iter().find(|field| field.name == name.as_ref())?;
                if path.is_empty() {
                    Some(field)
                } else {
                    self.find_field_path(path)
                }
            }
        }
    }

    pub fn size(&self) -> usize {
        self.flat.len()
    }

    pub fn field_is_scalar<Extensions>(&self, field: &Field<Extensions, Input>) -> bool {
        self.index.type_is_scalar(field.type_of.name())
    }

    pub fn field_is_enum<Extensions>(&self, field: &Field<Extensions, Input>) -> bool {
        self.index.type_is_enum(field.type_of.name())
    }

    pub fn field_validate_enum_value<Extensions>(
        &self,
        field: &Field<Extensions, Input>,
        value: &str,
    ) -> bool {
        self.index.validate_enum_value(field.type_of.name(), value)
    }

    pub fn field_nested_iter<'a, Output>(
        &'a self,
        field: &'a Field<Nested<Input>, Input>,
        value: &'a Output,
    ) -> impl Iterator<Item = &'a Field<Nested<Input>, Input>>
    where
        Output: TypedValue<'a>,
    {
        let value_type = field.value_type(value);

        field.nested_iter(move |field| {
            self.index
                .is_type_implements(value_type, &field.type_condition)
        })
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
        map: impl Fn(Input) -> Result<Output, Error>,
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
#[derive(
    Debug, PartialOrd, Ord, PartialEq, Eq, Clone, Copy, Default, Hash, Serialize, Deserialize,
)]
pub struct Pos {
    /// One-based line number.
    pub line: usize,

    /// One-based column number.
    pub column: usize,
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
pub enum PathSegment {
    /// A field in an object.
    Field(String),
    /// An index in a list.
    Index(usize),
}

impl From<async_graphql::PathSegment> for PathSegment {
    fn from(value: async_graphql::PathSegment) -> Self {
        match value {
            async_graphql::PathSegment::Field(field) => PathSegment::Field(field),
            async_graphql::PathSegment::Index(index) => PathSegment::Index(index),
        }
    }
}

impl From<PathSegment> for async_graphql::PathSegment {
    fn from(val: PathSegment) -> Self {
        match val {
            PathSegment::Field(field) => async_graphql::PathSegment::Field(field),
            PathSegment::Index(index) => async_graphql::PathSegment::Index(index),
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct Positioned<Value> {
    pub value: Value,
    pub pos: Pos,
    pub path: Vec<PathSegment>,
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
    pub fn with_path(&mut self, path: Vec<PathSegment>) -> Self {
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

impl From<Positioned<Error>> for ServerError {
    fn from(val: Positioned<Error>) -> Self {
        match val.value {
            Error::ServerError(e) => e,
            _ => {
                let extensions = val.value.extend().extensions;
                let mut server_error =
                    ServerError::new(val.value.to_string(), Some(val.pos.into()));

                server_error.extensions = extensions;

                // TODO: in order to be compatible with async_graphql path is only set for
                // validation errors here but in general we might consider setting it
                // for every error
                if let Error::Validation(_) = val.value {
                    server_error.path = val
                        .path
                        .into_iter()
                        .map(|path| path.into())
                        .collect::<Vec<_>>();
                }

                server_error
            }
        }
    }
}

#[cfg(test)]
mod test {
    use async_graphql::parser::types::ConstDirective;
    use async_graphql_value::ConstValue;

    use super::Directive;

    #[test]
    fn test_from_custom_directive() {
        let custom_directive = Directive {
            name: "options".to_string(),
            arguments: vec![("paging".to_string(), ConstValue::Boolean(true))],
        };

        let async_directive: ConstDirective = (&custom_directive).into();
        insta::assert_debug_snapshot!(async_directive);
    }
}
