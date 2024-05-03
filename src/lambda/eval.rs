use core::future::Future;
use std::pin::Pin;

use super::{Concurrent, EvaluationContext, EvaluationError, ResolverContextLike};

pub trait Eval<Output = async_graphql::Value>
where
    Self: Send + Sync,
{
    fn eval<'a, Ctx: ResolverContextLike<'a> + Sync + Send>(
        &'a self,
        ctx: EvaluationContext<'a, Ctx>,
        conc: &'a Concurrent,
    ) -> Pin<Box<dyn Future<Output = Result<Output, EvaluationError>> + 'a + Send>>
    where
        Output: 'a;
}
