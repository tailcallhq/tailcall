use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

use async_graphql::parser::types::{ConstDirective, OperationType};
use async_graphql::Pos;
use async_graphql_value::ConstValue;
use serde::Deserialize;

use crate::core::ir::model::IR;

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
    pub name: String,
    pub ir: Option<IR>,
    pub type_of: crate::core::blueprint::Type,
    pub skip: Option<Variable>,
    pub include: Option<Variable>,
    pub args: Vec<Arg<Input>>,
    pub extensions: Option<Extensions>,
    pub pos: Pos,
    pub is_scalar: bool,
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
    pub fn try_map<Output, Error>(
        self,
        map: impl Fn(Input) -> Result<Output, Error>,
    ) -> Result<Field<Extensions, Output>, Error> {
        Ok(Field {
            id: self.id,
            name: self.name,
            ir: self.ir,
            type_of: self.type_of,
            extensions: self.extensions,
            skip: self.skip,
            include: self.include,
            pos: self.pos,
            args: self
                .args
                .into_iter()
                .map(|arg| arg.try_map(&map))
                .collect::<Result<_, _>>()?,
            is_scalar: self.is_scalar,
            directives: self
                .directives
                .into_iter()
                .map(|directive| directive.try_map(&map))
                .collect::<Result<_, _>>()?,
        })
    }
}

impl<Input> Field<Nested<Input>, Input> {
    pub fn nested(&self) -> Option<&Vec<Field<Nested<Input>, Input>>> {
        self.extensions.as_ref().map(|Nested(nested)| nested)
    }

    pub fn nested_iter(&self) -> impl Iterator<Item = &Field<Nested<Input>, Input>> {
        self.nested()
            .map(|nested| nested.iter())
            .into_iter()
            .flatten()
    }
}

impl<Input> Field<Flat, Input> {
    fn parent(&self) -> Option<&FieldId> {
        self.extensions.as_ref().map(|Flat(id)| id)
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
            ir: self.ir,
            type_of: self.type_of,
            skip: self.skip,
            include: self.include,
            args: self.args,
            pos: self.pos,
            extensions,
            is_scalar: self.is_scalar,
            directives: self.directives,
        }
    }
}

impl<Extensions: Debug, Input: Debug> Debug for Field<Extensions, Input> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut debug_struct = f.debug_struct("Field");
        debug_struct.field("id", &self.id);
        debug_struct.field("name", &self.name);
        if self.ir.is_some() {
            debug_struct.field("ir", &"Some(..)");
        }
        debug_struct.field("type_of", &self.type_of);
        if !self.args.is_empty() {
            debug_struct.field("args", &self.args);
        }
        if self.extensions.is_some() {
            debug_struct.field("extensions", &self.extensions);
        }
        debug_struct.field("is_scalar", &self.is_scalar);
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
#[derive(Clone)]
pub struct Flat(FieldId);

impl Flat {
    pub fn new(id: FieldId) -> Self {
        Flat(id)
    }
    pub fn as_id(&self) -> &FieldId {
        &self.0
    }
}
impl Debug for Flat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Flat({:?})", self.0)
    }
}

/// Store field relationships in a nested structure like a tree where each field
/// links to its children.
#[derive(Clone, Debug)]
pub struct Nested<Input>(Vec<Field<Nested<Input>, Input>>);

#[derive(Clone, Debug)]
pub struct OperationPlan<Input> {
    flat: Vec<Field<Flat, Input>>,
    operation_type: OperationType,
    nested: Vec<Field<Nested<Input>, Input>>,
}

impl<Input> OperationPlan<Input> {
    pub fn new(fields: Vec<Field<Flat, Input>>, operation_type: OperationType) -> Self
    where
        Input: Clone,
    {
        let nested = fields
            .clone()
            .into_iter()
            .filter(|f| f.extensions.is_none())
            .map(|f| f.into_nested(&fields))
            .collect::<Vec<_>>();

        Self { flat: fields, nested, operation_type }
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
}

/// DirectiveAdapter helps to bridge the gap between async_graphql's
/// ConstDirective and our custom Directive implementation.
pub trait DirectiveAdapter {
    fn name(&self) -> &str;
    fn arguments(&self) -> impl Iterator<Item = (&str, &ConstValue)>;
}

impl DirectiveAdapter for ConstDirective {
    fn name(&self) -> &str {
        &self.name.node
    }

    fn arguments(&self) -> impl Iterator<Item = (&str, &ConstValue)> {
        self.arguments
            .iter()
            .map(|(k, v)| (k.node.as_str(), &v.node))
    }
}

impl DirectiveAdapter for Directive<ConstValue> {
    fn name(&self) -> &str {
        &self.name
    }

    fn arguments(&self) -> impl Iterator<Item = (&str, &ConstValue)> {
        self.arguments.iter().map(|(k, v)| (k.as_str(), v))
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
