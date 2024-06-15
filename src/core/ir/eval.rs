use std::future::Future;

use super::{EvaluationContext, EvaluationError, ResolverContextLike};

pub trait Eval<Output = async_graphql::Value> {
    fn eval<Ctx>(
        &self,
        ctx: &mut EvaluationContext<'_, Ctx>,
    ) -> impl Future<Output = Result<Output, EvaluationError>>
    where
        Ctx: ResolverContextLike + Sync;
}
