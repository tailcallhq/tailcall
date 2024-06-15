use std::future::Future;
use super::{EvaluationContext, EvaluationError, ResolverContextLike};

pub trait Eval<Output = async_graphql::Value>
where
    Self: Send + Sync,
{
    fn eval<'slf, 'ctx, Ctx>(
        &'slf self,
        ctx: &'ctx mut EvaluationContext<'slf, Ctx>,
    ) -> impl Future<Output = Result<Output, EvaluationError>>
    where
        Ctx: ResolverContextLike<'slf> + Sync + Send;
}
