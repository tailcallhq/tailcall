use std::convert::Infallible;

use tailcall_valid::Valid;

use crate::core::ir::model::IO;
use crate::core::jit::{Field, OperationPlan};
use crate::core::Transform;

pub struct CheckDirectLoader<A>(std::marker::PhantomData<A>);
impl<A> CheckDirectLoader<A> {
    pub fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}

fn mark_direct_loader<A>(selection: &mut [Field<A>], is_parent_list: bool) {
    for field in selection.iter_mut() {
        if let Some(ir) = &mut field.ir {
            ir.modify_io(&mut |io| {
                if let IO::Http { use_batcher, .. } = io {
                    if is_parent_list {
                        field.use_batch_loader = Some(true);
                        *use_batcher = true;
                    }
                }
            });
        }
        mark_direct_loader(&mut field.selection, field.type_of.is_list());
    }
}

impl<A> Transform for CheckDirectLoader<A> {
    type Value = OperationPlan<A>;
    type Error = Infallible;

    fn transform(&self, mut plan: Self::Value) -> Valid<Self::Value, Self::Error> {
        mark_direct_loader(&mut plan.selection, false);
        Valid::succeed(plan)
    }
}
