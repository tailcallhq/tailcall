use std::marker::PhantomData;

use crate::core::jit::OperationPlan;
use crate::core::valid::Valid;
use crate::core::Transform;

pub struct CheckConst<A>(PhantomData<A>);
impl<A> CheckConst<A> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<A> Transform for CheckConst<A> {
    type Value = OperationPlan<A>;
    type Error = ();

    fn transform(&self, mut plan: Self::Value) -> Valid<Self::Value, Self::Error> {
        let is_const = plan.as_flat().iter().all(|field| match field.ir {
            Some(ref ir) => ir.is_const(),
            None => true,
        });

        plan.is_const = is_const;

        Valid::succeed(plan)
    }
}
