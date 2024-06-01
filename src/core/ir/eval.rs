use core::future::Future;
use std::pin::Pin;
use crate::core::ConstValue;

use super::{EvaluationContext, EvaluationError, ResolverContextLike};

pub trait Eval<Output = ConstValue>
where
    Self: Send + Sync,
{
    fn eval<'a, Ctx: ResolverContextLike<'a> + Sync + Send>(
        &'a self,
        ctx: EvaluationContext<'a, Ctx>,
    ) -> Pin<Box<dyn Future<Output = Result<Output, EvaluationError>> + 'a + Send>>
    where
        Output: 'a;
}
