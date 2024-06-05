use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

use async_graphql::parser::types::{DocumentOperations, ExecutableDocument, Selection};
use async_graphql::Positioned;

use super::field_index::{FieldIndex, QueryField};
use crate::core::blueprint::Blueprint;
use crate::core::ir::IR;
use crate::core::merge_right::MergeRight;

#[allow(unused)]
trait IncrGen {
    fn gen(&mut self) -> Self;
}

#[allow(unused)]
#[derive(Debug)]
pub enum Type {
    Named(String),
    List(Box<Type>),
    Required(Box<Type>),
}

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

impl IncrGen for ArgId {
    fn gen(&mut self) -> Self {
        let id = self.0;
        self.0 += 1;
        Self(id)
    }
}

#[allow(unused)]
impl ArgId {
    pub fn new(id: usize) -> Self {
        ArgId(id)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct FieldId(usize);

impl Debug for FieldId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[allow(unused)]
impl FieldId {
    pub fn new(id: usize) -> Self {
        FieldId(id)
    }
}

impl IncrGen for FieldId {
    fn gen(&mut self) -> Self {
        let id = self.0;
        self.0 += 1;
        Self(id)
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

    pub fn into_children(self, e: &ExecutionPlan) -> Field<Children> {
        let mut children = Vec::new();
        for field in e.fields.iter() {
            if let Some(id) = field.parent() {
                if *id == self.id {
                    children.push(field.to_owned().into_children(e));
                }
            }
        }

        Field {
            id: self.id,
            name: self.name,
            ir: self.ir,
            type_of: self.type_of,
            args: self.args,
            refs: Some(Children(children)),
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
pub struct Parent(pub(crate) FieldId);
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

#[derive(Debug, Clone)]
#[allow(unused)]
pub struct Children(pub(crate) Vec<Field<Children>>);

#[derive(Clone, Debug)]
pub struct ExecutionPlan {
    pub fields: Vec<Field<Parent>>,
}

#[allow(unused)]
pub struct ExecutionPlanBuilder {
    index: FieldIndex,
}

impl ExecutionPlan {
    #[allow(unused)]
    pub fn into_children(self) -> Vec<Field<Children>> {
        let this = &self.clone();
        let fields = self.fields.into_iter();

        fields.map(|f| f.into_children(this)).collect::<Vec<_>>()
    }

    #[allow(unused)]
    pub fn find_field(&self, id: FieldId) -> Option<&Field<Parent>> {
        self.fields.iter().find(|field| field.id == id)
    }
}

impl ExecutionPlanBuilder {
    #[allow(unused)]
    pub fn new(blueprint: Blueprint) -> Self {
        let blueprint_index = FieldIndex::init(&blueprint);
        Self { index: blueprint_index }
    }

    #[allow(unused)]
    pub fn build(&self, document: ExecutableDocument) -> anyhow::Result<ExecutionPlan> {
        let fields = self.create_field_set(document)?;
        Ok(ExecutionPlan { fields })
    }

    #[allow(clippy::too_many_arguments)]
    fn resolve_selection_set(
        &self,
        selection_set: Positioned<async_graphql_parser::types::SelectionSet>,
        id: &mut FieldId,
        arg_id: &mut ArgId,
        current_type: &str,
        parent: Option<Parent>,
    ) -> anyhow::Result<Vec<Field<Parent>>> {
        let mut fields = Vec::new();

        for selection in selection_set.node.items {
            if let Selection::Field(gql_field) = selection.node {
                let field_name = gql_field.node.name.node.as_str();
                let field_args = gql_field
                    .node
                    .arguments
                    .into_iter()
                    .map(|(k, v)| (k.node.as_str().to_string(), v.node))
                    .collect::<HashMap<_, _>>();

                if let Some(field_def) = self.index.get_field(current_type, field_name) {
                    let mut args = vec![];
                    for (arg_name, value) in field_args {
                        if let Some(arg) = field_def.get_arg(&arg_name) {
                            let type_of = arg.of_type.clone();
                            let id = arg_id.gen();
                            let arg = Arg {
                                id,
                                name: arg_name.clone(),
                                type_of,
                                value: Some(value),
                                default_value: arg
                                    .default_value
                                    .as_ref()
                                    .and_then(|v| v.to_owned().try_into().ok()),
                            };
                            args.push(arg);
                        }
                    }

                    let type_of = match field_def {
                        QueryField::Field((field_def, _)) => field_def.of_type.clone(),
                        QueryField::InputField(field_def) => field_def.of_type.clone(),
                    };

                    let cur_id = id.gen();
                    let child_fields = self.resolve_selection_set(
                        gql_field.node.selection_set.clone(),
                        id,
                        arg_id,
                        type_of.name(),
                        Some(Parent(cur_id.clone())),
                    )?;
                    let field = Field {
                        id: cur_id,
                        name: field_name.to_string(),
                        ir: match field_def {
                            QueryField::Field((field_def, _)) => field_def.resolver.clone(),
                            _ => None,
                        },
                        type_of,
                        args,
                        refs: parent.clone(),
                    };

                    fields.push(field);
                    fields = fields.merge_right(child_fields);
                }
            }
        }

        Ok(fields)
    }

    fn create_field_set(&self, document: ExecutableDocument) -> anyhow::Result<Vec<Field<Parent>>> {
        let query = self.index.get_query();
        let mut id = FieldId::new(0);
        let mut arg_id = ArgId::new(0);

        let mut fields = Vec::new();

        for (_, fragment) in document.fragments {
            fields = self.resolve_selection_set(
                fragment.node.selection_set,
                &mut id,
                &mut arg_id,
                query,
                None,
            )?;
        }

        match document.operations {
            DocumentOperations::Single(single) => {
                fields = self.resolve_selection_set(
                    single.node.selection_set,
                    &mut id,
                    &mut arg_id,
                    query,
                    None,
                )?;
            }
            DocumentOperations::Multiple(multiple) => {
                for (_, single) in multiple {
                    fields = self.resolve_selection_set(
                        single.node.selection_set,
                        &mut id,
                        &mut arg_id,
                        query,
                        None,
                    )?;
                }
            }
        }

        Ok(fields)
    }
}
