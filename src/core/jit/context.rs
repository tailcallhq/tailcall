use async_graphql::{Name, SelectionField, ServerError};
use async_graphql_value::ConstValue;
use derive_getters::Getters;

use super::Request;
use crate::core::ir::ResolverContextLike;

/// Rust representation of the GraphQL context available in the DSL
#[derive(Getters, Debug, Clone)]
pub struct Context<'a, Input, Output> {
    request: &'a Request<Input>,
    value: Option<&'a Output>,
    args: Option<indexmap::IndexMap<Name, Input>>,
}

impl<'a, Input, Output> Context<'a, Input, Output> {
    pub fn new(request: &'a Request<Input>) -> Self {
        Self { request, value: None, args: None }
    }

    pub fn with_value(&self, value: &'a Output) -> Self {
        Self { request: self.request, value: Some(value), args: None }
    }

    pub fn with_args(&self, args: indexmap::IndexMap<&str, Input>) -> Self {
        let mut map = indexmap::IndexMap::new();
        for (key, value) in args {
            map.insert(Name::new(key), value);
        }
        Self { request: self.request, value: self.value, args: Some(map) }
    }
}

impl<'a> ResolverContextLike for Context<'a, ConstValue, ConstValue> {
    fn value(&self) -> Option<&ConstValue> {
        self.value
    }

    // TODO: make generic over type of stored values
    fn args(&self) -> Option<&indexmap::IndexMap<Name, ConstValue>> {
        self.args.as_ref()
    }

    fn field(&self) -> Option<SelectionField> {
        todo!()
    }

    fn is_query(&self) -> bool {
        todo!()
    }

    fn add_error(&self, _error: ServerError) {
        todo!()
    }
}
