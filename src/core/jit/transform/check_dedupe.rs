use std::convert::Infallible;

use tailcall_valid::Valid;

use crate::core::ir::model::IR;
use crate::core::jit::OperationPlan;
use crate::core::Transform;

pub struct CheckDedupe<A>(std::marker::PhantomData<A>);
impl<A> CheckDedupe<A> {
    pub fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}

#[inline]
fn check_dedupe(ir: &IR) -> bool {
    match ir {
        IR::IO(io) => io.dedupe(),
        IR::Cache(cache) => cache.io.dedupe(),
        IR::Path(ir, _) => check_dedupe(ir),
        IR::Protect(_, ir) => check_dedupe(ir),
        IR::Pipe(ir, ir1) => check_dedupe(ir) && check_dedupe(ir1),
        IR::Merge(vec) => vec.iter().all(check_dedupe),
        IR::Discriminate(_, ir) => check_dedupe(ir),
        IR::Entity(hash_map) => hash_map.values().all(check_dedupe),
        IR::Dynamic(_) => true,
        IR::ContextPath(_) => true,
        IR::Map(_) => true,
        IR::Service(_) => true,
    }
}

impl<A> Transform for CheckDedupe<A> {
    type Value = OperationPlan<A>;
    type Error = Infallible;

    fn transform(&self, mut plan: Self::Value) -> Valid<Self::Value, Self::Error> {
        let dedupe = plan.selection.iter().all(|field| {
            if let Some(ir) = field.ir.as_ref() {
                check_dedupe(ir)
            } else {
                true
            }
        });

        plan.is_dedupe = dedupe;

        Valid::succeed(plan)
    }
}
