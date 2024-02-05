use core::future::Future;
use std::num::NonZeroU64;
use std::pin::Pin;
use std::rc::Rc;

use anyhow::Result;
use async_graphql_value::ConstValue;

use super::{Concurrent, Eval, EvaluationContext, Expression, ResolverContextLike, IO};

pub trait CacheKey<Ctx> {
    fn cache_key(&self, ctx: &Ctx) -> u64;
}

#[derive(Clone, Debug)]
pub struct Cached {
    pub max_age: NonZeroU64,
    pub expr: IO,
}

impl Cached {
    ///
    /// Wraps an expression with the cache primitive.
    /// Performance DFS on the cache on the expression and identifies all the IO nodes.
    /// Then wraps each IO node with the cache primitive.
    ///
    pub fn wrap(max_age: NonZeroU64, expr: Expression) -> Expression {
        expr.modify(Rc::new(move |expr| match expr {
            Expression::IO(io) => Some(Expression::Cached(Cached { max_age, expr: io.clone() })),
            _ => None,
        }))
    }
}

impl Eval for Cached {
    fn eval<'a, Ctx: ResolverContextLike<'a> + Sync + Send>(
        &'a self,
        ctx: &'a EvaluationContext<'a, Ctx>,
        conc: &'a Concurrent,
    ) -> Pin<Box<dyn Future<Output = Result<ConstValue>> + 'a + Send>> {
        Box::pin(async move {
            let key = self.expr.cache_key(ctx);
            if let Some(val) = ctx.req_ctx.cache.get(&key).await? {
                Ok(val)
            } else {
                let val = self.expr.eval(ctx, conc).await?;
                ctx.req_ctx
                    .cache
                    .set(key, val.clone(), self.max_age)
                    .await?;
                Ok(val)
            }
        })
    }
}
