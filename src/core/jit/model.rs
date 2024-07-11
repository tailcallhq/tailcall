use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

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
pub struct Arg {
    pub id: ArgId,
    pub name: String,
    pub type_of: crate::core::blueprint::Type,
    pub value: Option<async_graphql_value::Value>,
    pub default_value: Option<async_graphql_value::ConstValue>,
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
pub struct Field<Extensions> {
    pub id: FieldId,
    pub name: String,
    pub ir: Option<IR>,
    pub type_of: crate::core::blueprint::Type,
    pub skip: Option<Variable>,
    pub include: Option<Variable>,
    pub args: Vec<Arg>,
    pub extensions: Option<Extensions>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Variable(String);

impl Variable {
    pub fn new(name: &str) -> Self {
        Variable(name.to_string())
    }
}

impl<A> Field<A> {
    #[inline(always)]
    pub fn skip(&self, variables: &Variables<async_graphql_value::ConstValue>) -> bool {
        let skip = match &self.skip {
            Some(Variable(name)) => variables.get(name).map_or(false, |value| match value {
                async_graphql_value::ConstValue::Boolean(b) => *b,
                _ => false,
            }),
            None => false,
        };
        let include = match &self.include {
            Some(Variable(name)) => variables.get(name).map_or(true, |value| match value {
                async_graphql_value::ConstValue::Boolean(b) => *b,
                _ => true,
            }),
            None => true,
        };

        skip == include
    }
}

const EMPTY_VEC: &Vec<Field<Nested>> = &Vec::new();
impl Field<Nested> {
    pub fn nested(&self) -> &Vec<Field<Nested>> {
        match &self.extensions {
            Some(Nested(children)) => children,
            _ => EMPTY_VEC,
        }
    }
}

impl Field<Flat> {
    fn parent(&self) -> Option<&FieldId> {
        self.extensions.as_ref().map(|Flat(id)| id)
    }

    fn into_nested(self, fields: &[Field<Flat>]) -> Field<Nested> {
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
            extensions,
        }
    }
}

impl<A: Debug + Clone> Debug for Field<A> {
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
pub struct Nested(Vec<Field<Nested>>);

#[derive(Clone, Debug)]
pub struct ExecutionPlan {
    flat: Vec<Field<Flat>>,
    nested: Vec<Field<Nested>>,
}

impl ExecutionPlan {
    pub fn new(fields: Vec<Field<Flat>>) -> Self {
        let nested = fields
            .clone()
            .into_iter()
            .filter(|f| f.extensions.is_none())
            .map(|f| f.into_nested(&fields))
            .collect::<Vec<_>>();

        Self { flat: fields, nested }
    }

    pub fn as_nested(&self) -> &[Field<Nested>] {
        &self.nested
    }

    pub fn into_nested(self) -> Vec<Field<Nested>> {
        self.nested
    }

    pub fn as_parent(&self) -> &[Field<Flat>] {
        &self.flat
    }

    pub fn find_field(&self, id: FieldId) -> Option<&Field<Flat>> {
        self.flat.iter().find(|field| field.id == id)
    }

    pub fn find_field_path<S: AsRef<str>>(&self, path: &[S]) -> Option<&Field<Flat>> {
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
