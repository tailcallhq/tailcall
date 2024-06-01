use std::sync::Arc;

use async_graphql::context::SelectionField;
use async_graphql::ServerError;

use crate::core::{BorrowedValue, extend_lifetime_ref, FromValue};

pub trait ResolverContextLike<'a>: Clone {
    fn value(&'a self) -> Option<&'a BorrowedValue>;
    fn args(&'a self) -> Option<&'a Vec<(String, BorrowedValue)>>;
    fn field(&'a self) -> Option<SelectionField>;
    fn add_error(&'a self, error: ServerError);
}

#[derive(Clone)]
pub struct EmptyResolverContext;

impl<'a> ResolverContextLike<'a> for EmptyResolverContext {
    fn value(&'a self) -> Option<&'a BorrowedValue> {
        None
    }

    fn args(&'a self) -> Option<&'a Vec<(String, BorrowedValue)>> {
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
    fn value(&'a self) -> Option<&'a BorrowedValue> {
        self.inner.parent_value.as_value().map(|v| v.clone().into_bvalue()).map(|v|{
            println!("i: {:?}", v);
            let x = extend_lifetime_ref(&v);
            println!("i: {:?}", x);
            x
        })
    }

    fn args(&'a self) -> Option<&'a Vec<(String, BorrowedValue)>> {
        // Some(self.inner.args.as_index_map()) // TODO: FIXME
        None
    }

    fn field(&'a self) -> Option<SelectionField> {
        Some(self.inner.ctx.field())
    }

    fn add_error(&'a self, error: ServerError) {
        self.inner.ctx.add_error(error)
    }
}
