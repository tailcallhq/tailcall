use core::future::Future;
use std::pin::Pin;

use super::{EvaluationContext, EvaluationError, ResolverContextLike};

pub trait Eval<Output = async_graphql::Value>
where
    Self: Send + Sync,
{
    fn eval<'slf, 'ctx, Ctx: ResolverContextLike + Sync + Send>(
        &'slf self,
        ctx: &'ctx mut EvaluationContext<'slf, Ctx>,
    ) -> Pin<Box<dyn Future<Output = Result<Output, EvaluationError>> + 'ctx + Send>>;
}
