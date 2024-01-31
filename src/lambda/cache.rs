use core::future::Future;
use std::num::NonZeroU64;
use std::pin::Pin;
use std::sync::{Arc, RwLock};

use anyhow::Result;
use async_graphql_value::ConstValue;

use super::{Concurrent, Eval, EvaluationContext, Expression, ResolverContextLike, IO};
use crate::cache_key::CacheKey;

#[derive(Clone, Debug)]
pub enum Cached {
    IOCache(IOCache),
    NonIOCache(NonIOCache),
}

#[derive(Clone, Debug)]
pub struct IOCache {
    max_age: NonZeroU64,
    expr: Box<Expression>,
}

#[derive(Clone, Debug)]
pub struct NonIOCache {
    data: Arc<RwLock<Option<ConstValue>>>,
    expr: Box<Expression>,
}

impl Cached {
    pub fn new(max_age: NonZeroU64, expr: Expression) -> Self {
        match &expr {
            Expression::IO(_) => Cached::IOCache(IOCache { max_age, expr: Box::new(expr) }),
            _ => Cached::NonIOCache(NonIOCache {
                data: Arc::new(RwLock::new(None)),
                expr: Box::new(expr),
            }),
        }
    }
}

impl Eval for Cached {
    fn eval<'a, Ctx: ResolverContextLike<'a> + Sync + Send>(
        &'a self,
        ctx: &'a EvaluationContext<'a, Ctx>,
        conc: &'a Concurrent,
    ) -> Pin<Box<dyn Future<Output = Result<ConstValue>> + 'a + Send>> {
        Box::pin(async move {
            match self {
                Cached::IOCache(IOCache { max_age, expr }) => {
                    let key = match expr.as_ref() {
                        Expression::IO(io) => match io {
                            IO::Http { req_template, .. } => req_template.cache_key(ctx),
                            IO::Grpc { req_template, .. } => req_template.cache_key(ctx),
                            IO::GraphQLEndpoint { req_template, .. } => req_template.cache_key(ctx),
                        },
                        _ => Err(anyhow::anyhow!(
                            "IOCache shouldn't contain non-IO expressions"
                        )),
                    }?;

                    if let Some(val) = ctx.req_ctx.cache.get(&key).await? {
                        Ok(val)
                    } else {
                        let val = expr.eval(ctx, conc).await?;
                        ctx.req_ctx.cache.set(key, val.clone(), *max_age).await?;
                        Ok(val)
                    }
                }
                Cached::NonIOCache(NonIOCache { data, expr }) => {
                    let cache_lookup = data.read().unwrap().clone();
                    if let Some(val) = cache_lookup {
                        Ok(val)
                    } else {
                        let val = expr.eval(ctx, conc).await?;
                        *data.write().unwrap() = Some(val.clone());
                        Ok(val)
                    }
                }
            }
        })
    }
}
