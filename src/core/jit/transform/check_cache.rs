use std::convert::Infallible;
use std::num::NonZeroU64;

use tailcall_valid::Valid;

use crate::core::ir::model::IR;
use crate::core::jit::OperationPlan;
use crate::core::Transform;

/// A transformer that sets the minimum cache TTL for the operation plan based
/// on the IR.
pub struct CheckCache<A>(std::marker::PhantomData<A>);
impl<A> CheckCache<A> {
    pub fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}

#[inline]
fn check_cache(ir: &IR) -> Option<NonZeroU64> {
    match ir {
        IR::IO(_) => None,
        IR::Cache(cache) => Some(cache.max_age),
        IR::Path(ir, _) => check_cache(ir),
        IR::Protect(_, ir) => check_cache(ir),
        IR::Pipe(ir, ir1) => match (check_cache(ir), check_cache(ir1)) {
            (Some(age1), Some(age2)) => Some(age1.min(age2)),
            _ => None,
        },
        IR::Merge(vec) => vec.iter().map(check_cache).min().unwrap_or_default(),
        IR::Discriminate(_, ir) => check_cache(ir),
        IR::Entity(hash_map) => hash_map.values().map(check_cache).min().unwrap_or_default(),
        IR::Dynamic(_) | IR::ContextPath(_) | IR::Map(_) | IR::Service(_) => None,
    }
}

impl<A> Transform for CheckCache<A> {
    type Value = OperationPlan<A>;
    type Error = Infallible;

    fn transform(&self, mut plan: Self::Value) -> Valid<Self::Value, Self::Error> {
        let mut ttl = Some(NonZeroU64::MAX);

        for field in plan.selection.iter() {
            if let Some(ir) = field.ir.as_ref() {
                ttl = std::cmp::min(ttl, check_cache(ir));
            }
        }

        plan.min_cache_ttl = ttl;

        Valid::succeed(plan)
    }
}
