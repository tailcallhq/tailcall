use core::future::Future;
use std::num::NonZeroU64;
use std::ops::Deref;
use std::pin::Pin;

use anyhow::Result;
use async_graphql_value::ConstValue;

use super::{Concurrent, Eval, EvaluationContext, Expression, ResolverContextLike};

pub trait CacheKey<Ctx> {
    fn cache_key(&self, ctx: &Ctx) -> u64;
}

#[derive(Clone, Debug)]
pub struct Cache {
    pub max_age: NonZeroU64,
    pub expr: Box<Expression>,
}

impl Cache {
    ///
    /// Wraps an expression with the cache primitive.
    /// Performance DFS on the cache on the expression and identifies all the IO
    /// nodes. Then wraps each IO node with the cache primitive.
    pub fn wrap(max_age: NonZeroU64, expr: Expression) -> Expression {
        expr.modify(move |expr| match expr {
            Expression::IO(_) => Some(Expression::Cache(Cache {
                max_age,
                expr: Box::new(expr.clone()),
            })),
            _ => None,
        })
    }
}

impl Eval for Cache {
    fn eval<'a, Ctx: ResolverContextLike<'a> + Sync + Send>(
        &'a self,
        ctx: EvaluationContext<'a, Ctx>,
        conc: &'a Concurrent,
    ) -> Pin<Box<dyn Future<Output = Result<ConstValue>> + 'a + Send>> {
        Box::pin(async move {
            if let Expression::IO(io) = self.expr.deref() {
                let key = io.cache_key(&ctx);

                if let Some(val) = ctx.req_ctx.runtime.cache.get(&key).await? {
                    Ok(val)
                } else {
                    let val = self.expr.eval(ctx.clone(), conc).await?;
                    ctx.req_ctx
                        .runtime
                        .cache
                        .set(key, val.clone(), self.max_age)
                        .await?;
                    Ok(val)
                }
            } else {
                Ok(self.expr.eval(ctx, conc).await?)
            }
        })
    }
}
