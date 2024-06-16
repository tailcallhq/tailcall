mod error;
mod eval;
mod evaluation_context;
mod execute_http;

mod io;
mod resolver_context_like;

pub mod model;
pub use error::*;
pub use eval::*;
pub use evaluation_context::EvaluationContext;
pub use resolver_context_like::{EmptyResolverContext, ResolverContext, ResolverContextLike};

pub trait GraphQLOperationContext {
    fn selection_set(&self) -> Option<String>;
}
