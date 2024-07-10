use std::fmt::{Debug, Formatter};

use crate::core::ir::model::IR;

#[derive(Debug, Clone)]
pub struct Arg<Input> {
    pub id: ArgId,
    pub name: String,
    pub type_of: crate::core::blueprint::Type,
    pub value: Option<Input>,
    pub default_value: Option<Input>,
}

impl<Input> Arg<Input> {
    pub fn map_value<Output, Error>(
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
// TODO: do we need Clone constraints?
pub struct Field<Extensions: Clone, Input: Clone> {
    pub id: FieldId,
    pub name: String,
    pub ir: Option<IR>,
    pub type_of: crate::core::blueprint::Type,
    pub args: Vec<Arg<Input>>,
    pub extensions: Option<Extensions>,
}

impl<A: Clone, Input: Clone> Field<A, Input> {
    pub fn map_args<Output: Clone, Error>(
        self,
        map: impl Fn(Arg<Input>) -> Result<Arg<Output>, Error>,
    ) -> Result<Field<A, Output>, Error> {
        Ok(Field {
            id: self.id,
            name: self.name,
            ir: self.ir,
            type_of: self.type_of,
            extensions: self.extensions,
            args: self.args.into_iter().map(map).collect::<Result<_, _>>()?,
        })
    }
}

impl<Input: Clone> Field<Nested<Input>, Input> {
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

impl<Input: Clone> Field<Flat, Input> {
    fn parent(&self) -> Option<&FieldId> {
        self.extensions.as_ref().map(|Flat(id)| id)
    }

    fn into_nested(self, fields: &[Field<Flat, Input>]) -> Field<Nested<Input>, Input> {
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
            args: self.args,
            extensions,
        }
    }
}

impl<A: Debug + Clone, Input: Clone + Debug> Debug for Field<A, Input> {
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
pub struct Nested<Input: Clone>(Vec<Field<Nested<Input>, Input>>);

#[derive(Clone, Debug)]
pub struct ExecutionPlan<Input: Clone> {
    flat: Vec<Field<Flat, Input>>,
    nested: Vec<Field<Nested<Input>, Input>>,
}

impl<Input: Clone> ExecutionPlan<Input> {
    pub fn new(fields: Vec<Field<Flat, Input>>) -> Self {
        let nested = fields
            .clone()
            .into_iter()
            .filter(|f| f.extensions.is_none())
            .map(|f| f.into_nested(&fields))
            .collect::<Vec<_>>();

        Self { flat: fields, nested }
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
