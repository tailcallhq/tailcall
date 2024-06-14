use core::future::Future;
use std::pin::Pin;

use serde_json_borrow::OwnedValue;

use super::{EvaluationContext, EvaluationError, IoId, ResolverContextLike};
use crate::core::ir::jit::Store;

pub trait Eval<Output = async_graphql::Value>
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

pub trait EvalSync<Output>
where
    Self: Send + Sync,
{
    fn eval_sync<'a, Ctx: ResolverContextLike<'a> + Sync + Send>(
        &'a self,
        store: &'a Store<IoId, OwnedValue>,
        ctx: EvaluationContext<'a, Ctx>,
    ) -> Result<Output, EvaluationError>;
}
