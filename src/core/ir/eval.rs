use std::future::Future;

use super::{Error, EvaluationContext, ResolverContextLike};

pub trait Eval<Output = async_graphql::Value> {
    fn eval<Ctx>(
        &self,
        ctx: &mut EvaluationContext<'_, Ctx>,
    ) -> impl Future<Output = Result<Output, Error>>
    where
        Ctx: ResolverContextLike + Sync;
}
