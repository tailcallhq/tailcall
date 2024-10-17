use crate::core::ir::model::IR;
use crate::core::jit::OperationPlan;
use crate::core::valid::Valid;
use crate::core::Transform;

pub struct CheckDedupe<A>(std::marker::PhantomData<A>);
impl<A> CheckDedupe<A> {
    pub fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<A> Transform for CheckDedupe<A> {
    type Value = OperationPlan<A>;
    type Error = ();

    fn transform(&self, mut plan: Self::Value) -> Valid<Self::Value, Self::Error> {
        let dedupe = plan.as_nested().iter().all(|field| {
            if let Some(ir) = field.ir.as_ref() {
                match ir {
                    IR::IO(io) => io.dedupe(),
                    IR::Cache(cache) => cache.io.dedupe(),
                    _ => true,
                }
            } else {
                true
            }
        });

        plan.dedupe = dedupe;

        Valid::succeed(plan)
    }
}
