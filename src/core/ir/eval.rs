use super::{EvaluationContext, EvaluationError, ResolverContextLike};
use std::future::Future;

pub trait Eval<Output = async_graphql::Value>
where
    Self: Send + Sync,
{
    fn eval<Ctx>(
        &self,
        ctx: &mut EvaluationContext<'_, Ctx>,
    ) -> impl Future<Output = Result<Output, EvaluationError>>
    where
        Ctx: ResolverContextLike + Send + Sync;
}
