mod discriminator;
mod error;
mod eval;
mod eval_context;
mod eval_http;
mod eval_io;
mod resolver_context_like;

pub mod model;
use std::collections::HashMap;
use std::ops::Deref;

pub use discriminator::*;
pub use error::*;
pub use eval_context::EvalContext;
pub use resolver_context_like::{EmptyResolverContext, ResolverContext, ResolverContextLike};

/// Contains all the nested fields that are resolved with current parent
/// resolver i.e. fields that don't have their own resolver and are resolved by
/// the ancestor
#[derive(Debug, Default, Clone)]
pub struct RelatedFields(pub HashMap<String, RelatedFields>);

impl Deref for RelatedFields {
    type Target = HashMap<String, RelatedFields>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub trait GraphQLOperationContext {
    fn directives(&self) -> Option<String>;
    fn selection_set(&self, related_fields: &RelatedFields) -> Option<String>;
}
