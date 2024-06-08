use core::future::Future;
use std::pin::Pin;

use super::{Error, EvaluationContext, ResolverContextLike};

pub trait Eval<Output = async_graphql::Value>
where
    Self: Send + Sync,
{
    fn eval<'a, Ctx: ResolverContextLike<'a> + Sync + Send>(
        &'a self,
        ctx: EvaluationContext<'a, Ctx>,
    ) -> Pin<Box<dyn Future<Output = Result<Output, Error>> + 'a + Send>>
    where
        Output: 'a;
}
