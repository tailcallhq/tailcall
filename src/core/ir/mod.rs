mod error;
mod eval;
mod eval_context;
mod eval_http;
mod eval_io;
mod jit;
mod resolver_context_like;

pub mod model;
pub use error::*;
pub use eval_context::EvalContext;
pub use resolver_context_like::{EmptyResolverContext, ResolverContext, ResolverContextLike};

pub trait GraphQLOperationContext {
    fn selection_set(&self) -> Option<String>;
}
