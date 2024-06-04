use std::sync::Arc;

use async_graphql::context::SelectionField;
use async_graphql::{Name, ServerError, Value};
use indexmap::IndexMap;

pub trait ResolverContextLike: Clone {
    fn value(&self) -> Option<&Value>;
    fn args(&self) -> Option<&IndexMap<Name, Value>>;
    fn field(&self) -> Option<SelectionField>;
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

impl<'a> ResolverContextLike for ResolverContext<'a> {
    fn value(&self) -> Option<&Value> {
        self.inner.parent_value.as_value()
    }

    fn args(&self) -> Option<&IndexMap<Name, Value>> {
        Some(self.inner.args.as_index_map())
    }

    fn field(&self) -> Option<SelectionField> {
        Some(self.inner.ctx.field())
    }

    fn add_error(&self, error: ServerError) {
        self.inner.ctx.add_error(error)
    }
}
