use core::future::Future;
use std::pin::Pin;

use anyhow::Result;

use super::{EvaluationContext, ResolverContextLike};

pub trait Eval<Output = async_graphql::Value>
where
    Self: Send + Sync,
{
    fn eval<'a, Ctx: ResolverContextLike<'a> + Sync + Send>(
        &'a self,
        ctx: EvaluationContext<'a, Ctx>,
    ) -> Pin<Box<dyn Future<Output = Result<Output>> + 'a + Send>>
    where
        Output: 'a;
}
