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
fn check_cacheable(ir: &IR) -> Option<NonZeroU64> {
    match ir {
        IR::IO(_) => None,
        IR::Cache(cache) => Some(cache.max_age),
        IR::Path(ir, _) => check_cacheable(ir),
        IR::Protect(ir) => check_cacheable(ir),
        IR::Pipe(ir, ir1) => match (check_cacheable(ir), check_cacheable(ir1)) {
            (Some(age1), Some(age2)) => Some(age1.min(age2)),
            _ => None,
        },
        IR::Discriminate(_, ir) => check_cacheable(ir),
        IR::Entity(hash_map) => {
            let mut cache_ttl = None;
            for ir in hash_map.values() {
                let result = check_cacheable(ir);
                if result.is_none() {
                    return None;
                }
                cache_ttl = match cache_ttl {
                    Some(max_age) => Some(std::cmp::min(max_age, result.unwrap())),
                    None => result,
                }
            }
            cache_ttl
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
                let result = check_cacheable(ir);
                if result.is_none() {
                    // not cacheable, break out of loop.
                    cache_ttl = None;
                    break;
                }
                cache_ttl = match cache_ttl {
                    Some(max_age) => Some(std::cmp::min(max_age, result.unwrap())),
                    None => result,
                };
            }
        }

        plan.cache_ttl = cache_ttl;

        Valid::succeed(plan)
    }
}
