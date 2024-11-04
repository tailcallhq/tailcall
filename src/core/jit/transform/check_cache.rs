use std::convert::Infallible;
use std::num::NonZeroU64;

use tailcall_valid::Valid;

use crate::core::ir::model::IR;
use crate::core::jit::OperationPlan;
use crate::core::Transform;

pub struct CheckCacheable<A>(std::marker::PhantomData<A>);
impl<A> CheckCacheable<A> {
    pub fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}

#[inline]
fn check_cacheble(ir: &IR) -> Option<NonZeroU64> {
    match ir {
        IR::IO(_) => None,
        IR::Cache(cache) => Some(cache.max_age),
        IR::Path(ir, _) => check_cacheble(ir),
        IR::Protect(ir) => check_cacheble(ir),
        IR::Pipe(ir, ir1) => {
            let result1 = check_cacheble(ir);
            let result2 = check_cacheble(ir1);

            if result1.is_some() && result2.is_some() {
                Some(std::cmp::min(result1.unwrap(), result2.unwrap()))
            } else {
                None
            }
        }
        IR::Discriminate(_, ir) => check_cacheble(ir),
        IR::Entity(hash_map) => {
            let mut final_result = None;
            for ir in hash_map.values() {
                let result = check_cacheble(ir);
                if result.is_none() {
                    return None;
                }
                final_result = match final_result {
                    Some(min_age) => Some(std::cmp::min(min_age, result.unwrap())),
                    None => result,
                };
            }
            final_result
        }
        IR::Dynamic(_) | IR::ContextPath(_) | IR::Map(_) | IR::Service(_) => None,
    }
}

impl<A> Transform for CheckCacheable<A> {
    type Value = OperationPlan<A>;
    type Error = Infallible;

    fn transform(&self, mut plan: Self::Value) -> Valid<Self::Value, Self::Error> {
        let mut cache_ttl: Option<NonZeroU64> = None;

        for field in plan.selection.iter() {
            if let Some(ir) = field.ir.as_ref() {
                let result = check_cacheble(ir);
                if result.is_none() {
                    // not cacheable, break out of loop.
                    cache_ttl = None;
                    break;
                }
                // fix None case for final_result.
                cache_ttl = Some(std::cmp::min(cache_ttl.unwrap(), result.unwrap()));
            }
        }

        plan.cache_ttl = cache_ttl;

        Valid::succeed(plan)
    }
}
