use derive_getters::Getters;

use super::Request;

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
