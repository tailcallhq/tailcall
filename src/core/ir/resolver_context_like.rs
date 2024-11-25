use std::sync::Arc;

use async_graphql::parser::types::{ConstDirective, OperationType};
use async_graphql::{Name, ServerError, Value};
use async_graphql_value::ConstValue;
use indexmap::IndexMap;

pub trait ResolverContextLike: Clone {
    fn value(&self) -> Option<&Value>;
    fn args(&self) -> Option<&IndexMap<Name, Value>>;
    fn field(&self) -> Option<SelectionField>;
    fn is_query(&self) -> bool;
    fn add_error(&self, error: ServerError);
}

#[derive(Clone)]
pub struct EmptyResolverContext;

impl ResolverContextLike for EmptyResolverContext {
    fn value(&self) -> Option<&Value> {
        None
    }

    fn args(&self) -> Option<&IndexMap<Name, Value>> {
        None
    }

    fn field(&self) -> Option<SelectionField> {
        None
    }

    fn is_query(&self) -> bool {
        false
    }

    fn add_error(&self, _: ServerError) {}
}

#[derive(Clone)]
pub struct ResolverContext<'a> {
    inner: Arc<async_graphql::dynamic::ResolverContext<'a>>,
}

impl<'a> From<async_graphql::dynamic::ResolverContext<'a>> for ResolverContext<'a> {
    fn from(value: async_graphql::dynamic::ResolverContext<'a>) -> Self {
        ResolverContext { inner: Arc::new(value) }
    }
}

impl ResolverContextLike for ResolverContext<'_> {
    fn value(&self) -> Option<&Value> {
        self.inner.parent_value.as_value()
    }

    fn args(&self) -> Option<&IndexMap<Name, Value>> {
        Some(self.inner.args.as_index_map())
    }

    fn field(&self) -> Option<SelectionField> {
        Some(SelectionField::from(self.inner.ctx.field()))
    }

    fn is_query(&self) -> bool {
        self.inner.ctx.query_env.operation.node.ty == OperationType::Query
    }

    fn add_error(&self, error: ServerError) {
        self.inner.ctx.add_error(error)
    }
}

#[derive(Debug)]
pub struct SelectionField {
    name: String,
    args: Vec<(String, String)>,
    directives: Option<Vec<ConstDirective>>,
    selection_set: Vec<SelectionField>,
}

impl From<async_graphql::SelectionField<'_>> for SelectionField {
    fn from(value: async_graphql::SelectionField) -> Self {
        Self::from_async_selection_field(value)
    }
}

impl<'a> From<&'a crate::core::jit::Field<ConstValue>> for SelectionField {
    fn from(value: &'a crate::core::jit::Field<ConstValue>) -> Self {
        Self::from_jit_field(value)
    }
}

impl SelectionField {
    fn from_jit_field(field: &crate::core::jit::Field<ConstValue>) -> SelectionField {
        let name = field.output_name.to_string();
        let type_name = field.type_of.name();
        let selection_set = field
            .iter()
            .filter(|field| match &field.type_condition {
                Some(type_condition) => type_condition == type_name,
                None => true,
            })
            .map(Self::from_jit_field)
            .collect();
        let args = field
            .args
            .iter()
            .filter_map(|a| a.value.as_ref().map(|v| (a.name.to_owned(), v.to_string())))
            .collect::<Vec<_>>();

        SelectionField {
            name,
            args,
            directives: if field.directives.is_empty() {
                None
            } else {
                Some(
                    field
                        .directives
                        .iter()
                        .map(|d| d.into())
                        .collect::<Vec<ConstDirective>>(),
                )
            },
            selection_set,
        }
    }

    fn from_async_selection_field(field: async_graphql::SelectionField) -> SelectionField {
        let name = field.name().to_owned();
        let args = field
            .arguments()
            .map_err(|err| {
                tracing::warn!("Failed to resolve arguments for field {name}, due to error: {err}");
                err
            })
            .unwrap_or_default()
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect::<Vec<_>>();

        let selection_set = field
            .selection_set()
            .map(Self::from_async_selection_field)
            .collect();

        Self {
            name,
            args,
            selection_set,
            directives: field.directives().ok(),
        }
    }

    pub fn directives(&self) -> &Option<Vec<ConstDirective>> {
        &self.directives
    }

    pub fn arguments(&self) -> &[(String, String)] {
        &self.args
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns an iterator over the `selection_set` that yields
    /// `SelectionField` instances.
    pub fn selection_set(&self) -> std::slice::Iter<SelectionField> {
        self.selection_set.iter()
    }
}
