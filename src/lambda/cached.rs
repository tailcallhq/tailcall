use core::future::Future;
use std::num::NonZeroU64;
use std::pin::Pin;

use anyhow::Result;
use async_graphql_value::ConstValue;

use super::{
    Concurrent, Eval, EvaluationContext, Expression, List, Logic, Math, Relation,
    ResolverContextLike, IO,
};

pub trait CacheKey<Ctx> {
    fn cache_key(&self, ctx: &Ctx) -> u64;
}

#[derive(Clone, Debug)]
pub struct Cached {
    pub max_age: NonZeroU64,
    pub expr: IO,
}

impl Cached {
    fn wrap_vec(max_age: NonZeroU64, exprs: Vec<Expression>) -> Vec<Expression> {
        exprs
            .into_iter()
            .map(|expr| Cached::wrap_ios(max_age, expr))
            .collect()
    }

    pub fn wrap_ios(max_age: NonZeroU64, expr: Expression) -> Expression {
        let box_cache_wrap = |max_age, expr| Box::new(Cached::wrap_ios(max_age, expr));
        // Expression::IO(io) => Expression::Cached(Cached { max_age, expr: io }),
        match expr {
            expr @ (Expression::Context(_) | Expression::Literal(_) | Expression::Cached(_)) => {
                expr
            }
            Expression::EqualTo(lhs, rhs) => {
                Expression::EqualTo(box_cache_wrap(max_age, *lhs), box_cache_wrap(max_age, *rhs))
            }
            Expression::IO(io) => Expression::Cached(Cached { max_age, expr: io }),
            Expression::Input(expr, path) => {
                Expression::Input(box_cache_wrap(max_age, *expr), path)
            }
            Expression::Logic(logic) => Expression::Logic(match logic {
                Logic::If { cond, then, els } => Logic::If {
                    cond: box_cache_wrap(max_age, *cond),
                    then: box_cache_wrap(max_age, *then),
                    els: box_cache_wrap(max_age, *els),
                },
                Logic::And(exprs) => Logic::And(Cached::wrap_vec(max_age, exprs)),
                Logic::Or(exprs) => Logic::Or(Cached::wrap_vec(max_age, exprs)),
                Logic::Cond(exprs) => Logic::Cond(
                    exprs
                        .into_iter()
                        .map(|(expr1, expr2)| {
                            (
                                box_cache_wrap(max_age, *expr1),
                                box_cache_wrap(max_age, *expr2),
                            )
                        })
                        .collect(),
                ),
                Logic::DefaultTo(expr1, expr2) => Logic::DefaultTo(
                    box_cache_wrap(max_age, *expr1),
                    box_cache_wrap(max_age, *expr2),
                ),
                Logic::IsEmpty(expr) => Logic::IsEmpty(box_cache_wrap(max_age, *expr)),
                Logic::Not(expr) => Logic::Not(expr),
            }),
            Expression::Relation(relation) => Expression::Relation(match relation {
                Relation::Intersection(exprs) => {
                    Relation::Intersection(Cached::wrap_vec(max_age, exprs))
                }
                Relation::Difference(expr1, expr2) => {
                    let expr1 = Cached::wrap_vec(max_age, expr1);
                    let expr2 = Cached::wrap_vec(max_age, expr2);
                    Relation::Difference(expr1, expr2)
                }
                Relation::Equals(lhs, rhs) => {
                    Relation::Equals(box_cache_wrap(max_age, *lhs), box_cache_wrap(max_age, *rhs))
                }
                Relation::Gt(lhs, rhs) => {
                    Relation::Gt(box_cache_wrap(max_age, *lhs), box_cache_wrap(max_age, *rhs))
                }
                Relation::Gte(lhs, rhs) => {
                    Relation::Gte(box_cache_wrap(max_age, *lhs), box_cache_wrap(max_age, *rhs))
                }
                Relation::Lt(lhs, rhs) => {
                    Relation::Lt(box_cache_wrap(max_age, *lhs), box_cache_wrap(max_age, *rhs))
                }
                Relation::Lte(lhs, rhs) => {
                    Relation::Lte(box_cache_wrap(max_age, *lhs), box_cache_wrap(max_age, *rhs))
                }
                Relation::Max(exprs) => Relation::Max(Cached::wrap_vec(max_age, exprs)),
                Relation::Min(exprs) => Relation::Min(Cached::wrap_vec(max_age, exprs)),
                Relation::PathEq(expr1, path, expr2) => Relation::PathEq(
                    box_cache_wrap(max_age, *expr1),
                    path,
                    box_cache_wrap(max_age, *expr2),
                ),
                Relation::PropEq(expr1, path, expr2) => Relation::PropEq(
                    box_cache_wrap(max_age, *expr1),
                    path,
                    box_cache_wrap(max_age, *expr2),
                ),
                Relation::SortPath(expr, path) => {
                    Relation::SortPath(box_cache_wrap(max_age, *expr), path)
                }
                Relation::SymmetricDifference(lhs, rhs) => Relation::SymmetricDifference(
                    Cached::wrap_vec(max_age, lhs),
                    Cached::wrap_vec(max_age, rhs),
                ),
                Relation::Union(lhs, rhs) => Relation::Union(
                    Cached::wrap_vec(max_age, lhs),
                    Cached::wrap_vec(max_age, rhs),
                ),
            }),
            Expression::List(list) => Expression::List(match list {
                List::Concat(exprs) => List::Concat(Cached::wrap_vec(max_age, exprs)),
            }),
            Expression::Math(math) => Expression::Math(match math {
                Math::Mod(lhs, rhs) => {
                    let lhs = box_cache_wrap(max_age, *lhs);
                    let rhs = box_cache_wrap(max_age, *rhs);
                    Math::Mod(lhs, rhs)
                }
                Math::Add(lhs, rhs) => {
                    let lhs = box_cache_wrap(max_age, *lhs);
                    let rhs = box_cache_wrap(max_age, *rhs);
                    Math::Add(lhs, rhs)
                }
                Math::Divide(lhs, rhs) => {
                    let lhs = box_cache_wrap(max_age, *lhs);
                    let rhs = box_cache_wrap(max_age, *rhs);
                    Math::Divide(lhs, rhs)
                }
                Math::Multiply(lhs, rhs) => {
                    let lhs = box_cache_wrap(max_age, *lhs);
                    let rhs = box_cache_wrap(max_age, *rhs);
                    Math::Multiply(lhs, rhs)
                }
                Math::Subtract(lhs, rhs) => {
                    let lhs = box_cache_wrap(max_age, *lhs);
                    let rhs = box_cache_wrap(max_age, *rhs);
                    Math::Subtract(lhs, rhs)
                }
                Math::Dec(expr) => Math::Dec(box_cache_wrap(max_age, *expr)),
                Math::Inc(expr) => Math::Inc(box_cache_wrap(max_age, *expr)),
                Math::Negate(expr) => Math::Negate(box_cache_wrap(max_age, *expr)),
                Math::Product(exprs) => Math::Product(Cached::wrap_vec(max_age, exprs)),
                Math::Sum(exprs) => Math::Sum(Cached::wrap_vec(max_age, exprs)),
            }),
            Expression::Concurrency(conc, expr) => {
                Expression::Concurrency(conc, box_cache_wrap(max_age, *expr))
            }
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
