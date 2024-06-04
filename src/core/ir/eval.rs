use core::future::Future;
use std::pin::Pin;

use super::{EvaluationContext, EvaluationError, ResolverContextLike};

pub trait Eval<Output = async_graphql::Value>
where
    Self: Send + Sync,
{
    fn eval<'a, 'b, Ctx: ResolverContextLike + Sync + Send>(
        &'a self,
        ctx: &'b mut EvaluationContext<'a, Ctx>,
    ) -> Pin<Box<dyn Future<Output = Result<Output, EvaluationError>> + 'b + Send>>;
}
