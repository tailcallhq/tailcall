use std::convert::Infallible;
use std::fmt::Debug;
use std::marker::PhantomData;

use tailcall_valid::Valid;

use crate::core::ir::model::IR;
use crate::core::jit::{Field, OperationPlan};
use crate::core::Transform;

pub struct DeferPlanner<A>(PhantomData<A>);

impl<A> DeferPlanner<A> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

// collect the fields that has IR type of Deferred and return back.
fn move_deferred_fields<A: Clone>(field: &mut Field<A>) -> Vec<Field<A>> {
    let mut deferred_fields = Vec::new();
    for selection in field.selection.iter_mut() {
        if let Some(IR::Deferred { .. }) = selection.ir {
            deferred_fields.push(selection.clone());
        }
        deferred_fields.extend(move_deferred_fields(selection));
    }

    field.selection.retain(|f| {
        f.ir.as_ref()
            .map_or(true, |ir| !matches!(ir, IR::Deferred { .. }))
    });

    deferred_fields
}

impl<A: Clone + Debug> Transform for DeferPlanner<A> {
    type Value = OperationPlan<A>;
    type Error = Infallible;

    fn transform(&self, mut plan: Self::Value) -> Valid<Self::Value, Self::Error> {
        let mut deferred_fields = Vec::new();
        for field in plan.selection.iter_mut() {
            if let Some(IR::Deferred { .. }) = field.ir {
                deferred_fields.push(field.clone());
            }

            deferred_fields.extend(move_deferred_fields(field));
        }

        plan.selection.retain(|f| {
            f.ir.as_ref()
                .map_or(true, |ir| !matches!(ir, IR::Deferred { .. }))
        });

        plan.deferred_fields = deferred_fields;

        Valid::succeed(plan)
    }
}
