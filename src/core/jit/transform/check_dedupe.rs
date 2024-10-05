use crate::core::{ir::model::IR, jit::OperationPlan, valid::Valid, Transform};

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
        let dedupe = plan
            .as_nested()
            .iter()
            .map(|field| {
                if let Some(IR::IO(io)) = field.ir.as_ref() {
                    io.dedupe()
                } else {
                    true
                }
            })
            .all(|a| a);

        plan.dedupe = dedupe;

        Valid::succeed(plan)
    }
}
