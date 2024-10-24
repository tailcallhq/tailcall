use std::convert::Infallible;
use std::marker::PhantomData;

use crate::core::ir::model::IR;
use crate::core::jit::OperationPlan;
use crate::core::valid::Valid;
use crate::core::Transform;

pub struct CheckProtected<A>(PhantomData<A>);
impl<A> CheckProtected<A> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

/// Checks if the IR will always evaluate to a Constant Value
pub fn is_protected(ir: &IR) -> bool {
    match ir {
        IR::Dynamic(dynamic_value) => false,
        IR::IO(_) => false,
        IR::Cache(_) => false,
        IR::Path(ir, _) => is_protected(ir),
        IR::ContextPath(_) => false,
        IR::Protect(_) => true,
        IR::Map(map) => is_protected(&map.input),
        IR::Pipe(ir, ir1) => is_protected(ir) || is_protected(ir1),
        IR::Discriminate(_, ir) => is_protected(ir),
        IR::Entity(hash_map) => hash_map.values().any(is_protected),
        IR::Service(_) => false,
    }
}

impl<A> Transform for CheckProtected<A> {
    type Value = OperationPlan<A>;
    type Error = Infallible;

    fn transform(&self, mut plan: Self::Value) -> Valid<Self::Value, Self::Error> {
        let is_protected = plan.iter_dfs().all(|field| match field.ir {
            Some(ref ir) => is_protected(ir),
            None => true,
        });

        plan.is_protected = is_protected;

        Valid::succeed(plan)
    }
}
