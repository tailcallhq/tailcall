use std::future::Future;
use super::{EvaluationContext, EvaluationError, ResolverContextLike};

pub trait Eval<Output = async_graphql::Value>
where
    Self: Send + Sync,
{
    fn eval<'a, Ctx>(
        &'a self,
        ctx: EvaluationContext<'a, Ctx>,
    ) -> impl Future<Output = Result<Output, EvaluationError>>
    where
        Ctx: ResolverContextLike<'a> + Sync + Send;
}
