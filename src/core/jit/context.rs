use derive_getters::Getters;

use super::Request;
use crate::core::ir::ResolverContextLike;

/// Rust representation of the GraphQL context available in the DSL
#[derive(Getters, Debug)]
pub struct Context<'a, Input, Output> {
    request: &'a Request<Input>,
    value: Option<&'a Output>,
    is_query: bool,
}

impl<'a, Input, Output> Clone for Context<'a, Input, Output> {
    fn clone(&self) -> Self {
        Self {
            request: self.request,
            value: self.value,
            is_query: self.is_query,
        }
    }
}

impl<'a, Input, Output> Context<'a, Input, Output> {
    pub fn new(request: &'a Request<Input>, is_query: bool) -> Self {
        Self { request, value: None, is_query }
    }

    pub fn with_value(&self, value: &'a Output) -> Self {
        Self {
            request: self.request,
            value: Some(value),
            is_query: self.is_query,
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
        self.is_query
    }

    fn add_error(&self, _error: async_graphql::ServerError) {
        todo!()
    }
}
