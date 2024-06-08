use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use std::rc::Rc;

use crate::core::counter::{Count, MutexCounter};
use crate::core::ir::IR;
use crate::core::CallId;

#[allow(unused)]
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
}

#[derive(Clone)]
pub struct Field<A: Clone> {
    pub id: FieldId,
    pub name: String,
    pub ir: Option<IR>,
    pub type_of: crate::core::blueprint::Type,
    pub args: Vec<Arg>,
    pub refs: Option<A>,
}

const EMPTY_VEC: &Vec<Field<Children>> = &Vec::new();
impl Field<Children> {
    #[allow(unused)]
    pub fn children(&self) -> &Vec<Field<Children>> {
        match &self.refs {
            Some(Children(children)) => children,
            _ => EMPTY_VEC,
        }
    }
}

impl Field<Parent> {
    pub fn parent(&self) -> Option<&FieldId> {
        self.refs.as_ref().map(|Parent(id)| id)
    }

    pub fn into_children(self, fields: &[Field<Parent>]) -> Field<Children> {
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
            refs,
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
        if self.refs.is_some() {
            debug_struct.field("refs", &self.refs);
        }
        debug_struct.finish()
    }
}

#[derive(Clone)]
pub struct Parent(FieldId);
#[allow(unused)]
impl Parent {
    pub fn new(id: FieldId) -> Self {
        Parent(id)
    }
}
impl Debug for Parent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parent({:?})", self.0)
    }
}

#[derive(Clone, Debug)]
pub struct Children(Vec<Field<Children>>);

#[derive(Clone, Debug)]
pub struct ExecutionPlan {
    fields: Vec<Field<Parent>>,
    field_children: Vec<Field<Children>>,
    counter: Rc<dyn Count<Item = CallId>>,
}

impl ExecutionPlan {
    pub fn new(fields: Vec<Field<Parent>>) -> Self {
        let fields_clone = fields.clone();
        let fields_clone = fields_clone.into_iter();
        let field_children = fields_clone
            .map(|f| f.into_children(&fields))
            .collect::<Vec<_>>();

        Self {
            fields,
            field_children,
            counter: Rc::new(MutexCounter::default()),
        }
    }

    pub fn counter(&self) -> &dyn Count<Item = CallId> {
        self.counter.deref()
    }

    #[allow(unused)]
    pub fn children(&self) -> &[Field<Children>] {
        &self.field_children
    }

    #[allow(unused)]
    pub fn parents(&self) -> &[Field<Parent>] {
        &self.fields
    }

    #[allow(unused)]
    pub fn find_field(&self, id: FieldId) -> Option<&Field<Parent>> {
        self.fields.iter().find(|field| field.id == id)
    }
}
