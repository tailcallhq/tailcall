use std::sync::Arc;

use derive_getters::Getters;

use super::Request;
use crate::core::ir::ResolverContextLike;

/// Rust representation of the GraphQL context available in the DSL
#[derive(Getters, Debug)]
pub struct Context<'a, Input, Output> {
    request: &'a Request<Input>,
    value: Option<&'a Output>,
    args: Option<&'a indexmap::IndexMap<&'a str, Input>>,
    arg_const_value: Option<Arc<indexmap::IndexMap<async_graphql::Name, async_graphql::Value>>>,
}

impl<'a, Input, Output> Clone for Context<'a, Input, Output> {
    fn clone(&self) -> Self {
        Self {
            request: self.request,
            value: self.value,
            args: self.args,
            arg_const_value: None,
        }
    }
}

impl<'a, Input, Output> Context<'a, Input, Output> {
    pub fn new(request: &'a Request<Input>) -> Self {
        Self { request, value: None, args: None, arg_const_value: None }
    }

    pub fn with_value(&self, value: &'a Output) -> Self {
        Self {
            request: self.request,
            value: Some(value),
            args: self.args,
            arg_const_value: None,
        }
    }

    pub fn with_args(&self, args: &'a indexmap::IndexMap<&str, Input>) -> Self
    where
        async_graphql::Value: From<Input>,
        Input: Clone,
    {
        let mut map = indexmap::IndexMap::new();
        for (key, value) in args.iter() {
            map.insert(async_graphql::Name::new(key), value.clone().into());
        }
        Self {
            request: self.request,
            value: self.value,
            args: Some(args),
            arg_const_value: Some(Arc::new(map)),
        }
    }
}

impl<'a> ResolverContextLike for Context<'a, async_graphql::Value, async_graphql::Value> {
    fn value(&self) -> Option<&async_graphql::Value> {
        self.value
    }

    fn args(&self) -> Option<&indexmap::IndexMap<async_graphql::Name, async_graphql::Value>> {
        self.arg_const_value.as_ref().map(Arc::as_ref)
    }

    fn field(&self) -> Option<async_graphql::SelectionField> {
        todo!()
    }

    fn is_query(&self) -> bool {
        todo!()
    }

    fn add_error(&self, _error: async_graphql::ServerError) {
        todo!()
    }
}
