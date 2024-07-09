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
pub struct Field<A: Clone, Input: Clone> {
    pub id: FieldId,
    pub name: String,
    pub ir: Option<IR>,
    pub type_of: crate::core::blueprint::Type,
    pub args: Vec<Arg<Input>>,
    pub extensions: Option<A>,
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

impl<Input: Clone> Field<Children<Input>, Input> {
    pub fn children(&self) -> Option<&Vec<Field<Children<Input>, Input>>> {
        self.extensions.as_ref().map(|Children(children)| children)
    }

    pub fn children_iter(&self) -> impl Iterator<Item = &Field<Children<Input>, Input>> {
        self.children()
            .map(|children| children.iter())
            .into_iter()
            .flatten()
    }
}

impl<Input: Clone> Field<Parent, Input> {
    fn parent(&self) -> Option<&FieldId> {
        self.extensions.as_ref().map(|Parent(id)| id)
    }

    fn into_children(self, fields: &[Field<Parent, Input>]) -> Field<Children<Input>, Input> {
        let mut children = Vec::new();
        for field in fields.iter() {
            if let Some(id) = field.parent() {
                if *id == self.id {
                    children.push(field.to_owned().into_children(fields));
                }
            }
        }

        let refs = if children.is_empty() {
            None
        } else {
            Some(Children(children))
        };

        Field {
            id: self.id,
            name: self.name,
            ir: self.ir,
            type_of: self.type_of,
            args: self.args,
            extensions: refs,
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
            debug_struct.field("refs", &self.extensions);
        }
        debug_struct.finish()
    }
}

#[derive(Clone)]
pub struct Parent(FieldId);

impl Parent {
    pub fn new(id: FieldId) -> Self {
        Parent(id)
    }
    pub fn as_id(&self) -> &FieldId {
        &self.0
    }
}
impl Debug for Parent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parent({:?})", self.0)
    }
}

#[derive(Clone, Debug)]
pub struct Children<Input: Clone>(Vec<Field<Children<Input>, Input>>);

#[derive(Clone, Debug)]
pub struct ExecutionPlan<Input: Clone> {
    parent: Vec<Field<Parent, Input>>,
    children: Vec<Field<Children<Input>, Input>>,
}

impl<Input: Clone> ExecutionPlan<Input> {
    pub fn new(fields: Vec<Field<Parent, Input>>) -> Self {
        let field_children = fields
            .clone()
            .into_iter()
            .filter(|f| f.extensions.is_none())
            .map(|f| f.into_children(&fields))
            .collect::<Vec<_>>();

        Self { parent: fields, children: field_children }
    }

    pub fn as_children(&self) -> &[Field<Children<Input>, Input>] {
        &self.children
    }

    pub fn into_children(self) -> Vec<Field<Children<Input>, Input>> {
        self.children
    }

    pub fn as_parent(&self) -> &[Field<Parent, Input>] {
        &self.parent
    }

    pub fn find_field(&self, id: FieldId) -> Option<&Field<Parent, Input>> {
        self.parent.iter().find(|field| field.id == id)
    }

    pub fn find_field_path<S: AsRef<str>>(&self, path: &[S]) -> Option<&Field<Parent, Input>> {
        match path.split_first() {
            None => None,
            Some((name, path)) => {
                let field = self
                    .parent
                    .iter()
                    .find(|field| field.name == name.as_ref())?;
                if path.is_empty() {
                    Some(field)
                } else {
                    self.find_field_path(path)
                }
            }
        }
    }

    pub fn size(&self) -> usize {
        self.parent.len()
    }
}
