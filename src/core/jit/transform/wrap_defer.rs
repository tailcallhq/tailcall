use std::{convert::Infallible, marker::PhantomData};

use tailcall_valid::Valid;

use crate::core::{
    ir::model::IR,
    jit::{Field, OperationPlan},
    Transform,
};

pub struct WrapDefer<A>(PhantomData<A>);

impl<A> WrapDefer<A> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

/// goes through selection and finds out IR's that needs to be deferred.
#[inline]
fn detect_and_wrap<A>(field: &mut Field<A>) {
    for selection in field.selection.iter_mut() {
        if let Some(ir) = std::mem::take(&mut selection.ir) {
            let ir = if selection
                .directives
                .iter()
                .find(|d| d.name == "defer")
                .is_some()
                && field.ir.is_some()
            {
                IR::Deferred(Box::new(ir))
            } else {
                ir
            };
            selection.ir = Some(ir);
        }
    }
}

impl<A> Transform for WrapDefer<A> {
    type Value = OperationPlan<A>;
    type Error = Infallible;
    fn transform(&self, mut plan: Self::Value) -> Valid<Self::Value, Self::Error> {
        plan.selection.iter_mut().for_each(detect_and_wrap);
        Valid::succeed(plan)
    }
}
