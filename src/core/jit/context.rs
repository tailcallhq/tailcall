use derive_getters::Getters;
use indexmap::IndexMap;

use super::Request;
use crate::core::ir::ResolverContextLike;

/// Rust representation of the GraphQL context available in the DSL
#[derive(Getters)]
pub struct Context<'a, Input, Output> {
    request: &'a Request<Input>,
    parent: Option<&'a Output>,
    value: Option<&'a Output>,
}

impl<'a, Input, Output> Clone for Context<'a, Input, Output> {
    fn clone(&self) -> Self {
        Self {
            request: self.request,
            parent: self.parent,
            value: self.value,
        }
    }
}

impl<'a, Input, Output> Context<'a, Input, Output> {
    pub fn new(request: &'a Request<Input>) -> Self {
        Self { request, parent: None, value: None }
    }

    pub fn with_parent_value(&self, value: &'a Output) -> Self {
        Self {
            request: self.request,
            parent: self.parent,
            value: Some(value),
        }
    }

    pub fn with_value(&self, value: &'a Output) -> Self {
        Self {
            request: self.request,
            parent: self.parent,
            value: Some(value),
        }
    }
}

impl<'a> ResolverContextLike for Context<'a, async_graphql::Value, async_graphql::Value> {
    fn value(&self) -> Option<&async_graphql::Value> {
        self.value
    }

    fn args(&self) -> Option<&indexmap::IndexMap<async_graphql::Name, async_graphql::Value>> {
        todo!()
    }

    fn field(&self) -> Option<async_graphql::SelectionField> {
        todo!()
    }

    fn is_query(&self) -> bool {
        todo!()
    }

    fn add_error(&self, error: async_graphql::ServerError) {
        todo!()
    }
}
