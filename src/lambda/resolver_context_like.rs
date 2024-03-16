use std::sync::Arc;

use async_graphql::context::SelectionField;
use async_graphql::{Name, ServerError, Value};
use indexmap::IndexMap;

pub trait ResolverContextLike<'a>: Clone {
    fn value(&'a self) -> Option<&'a Value>;
    fn args(&'a self) -> Option<&'a IndexMap<Name, Value>>;
    fn field(&'a self) -> Option<SelectionField>;
    fn add_error(&'a self, error: ServerError);
}

#[derive(Clone)]
pub struct EmptyResolverContext;

impl<'a> ResolverContextLike<'a> for EmptyResolverContext {
    fn value(&'a self) -> Option<&'a Value> {
        None
    }

    fn args(&'a self) -> Option<&'a IndexMap<Name, Value>> {
        None
    }

    fn field(&'a self) -> Option<SelectionField> {
        None
    }

    fn add_error(&'a self, _: ServerError) {}
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

impl<'a> ResolverContextLike<'a> for ResolverContext<'a> {
    fn value(&'a self) -> Option<&'a Value> {
        self.inner.parent_value.as_value()
    }

    fn args(&'a self) -> Option<&'a IndexMap<Name, Value>> {
        Some(self.inner.args.as_index_map())
    }

    fn field(&'a self) -> Option<SelectionField> {
        Some(self.inner.ctx.field())
    }

    fn add_error(&'a self, error: ServerError) {
        self.inner.ctx.add_error(error)
    }
}
