use std::convert::Infallible;
use std::marker::PhantomData;

use tailcall_valid::Valid;

use crate::core::ir::model::IR;
use crate::core::jit::OperationPlan;
use crate::core::Transform;

pub struct CheckConst<A>(PhantomData<A>);
impl<A> CheckConst<A> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

/// Checks if the IR will always evaluate to a Constant Value
pub fn is_const(ir: &IR) -> bool {
    match ir {
        IR::Dynamic(dynamic_value) => dynamic_value.is_const(),
        IR::IO(_) => false,
        IR::Cache(_) => false,
        IR::Path(ir, _) => is_const(ir),
        IR::ContextPath(_) => false,
        IR::Protect(_, ir) => is_const(ir),
        IR::Map(map) => is_const(&map.input),
        IR::Pipe(ir, ir1) => is_const(ir) && is_const(ir1),
        IR::Merge(vec) => vec.iter().all(is_const),
        IR::Discriminate(_, ir) => is_const(ir),
        IR::Entity(hash_map) => hash_map.values().all(is_const),
        IR::Service(_) => true,
    }
}

impl<A> Transform for CheckConst<A> {
    type Value = OperationPlan<A>;
    type Error = Infallible;

    fn transform(&self, mut plan: Self::Value) -> Valid<Self::Value, Self::Error> {
        let is_const = plan.iter_dfs().all(|field| match field.ir {
            Some(ref ir) => is_const(ir),
            None => true,
        });

        plan.is_const = is_const;

        Valid::succeed(plan)
    }
}
