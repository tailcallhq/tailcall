use std::convert::Infallible;

use tailcall_valid::Valid;

use crate::core::ir::model::IO;
use crate::core::jit::{Field, OperationPlan};
use crate::core::Transform;

pub struct CheckHttpMerge<A>(std::marker::PhantomData<A>);
impl<A> CheckHttpMerge<A> {
    pub fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}

// if from the root to the current field, in path there's list ancestor and
// current IR has group by clause then set use_batch_loader is true.
fn mark_direct_loader<A>(selection: &mut [Field<A>], has_list_ancestor: bool) {
    for field in selection.iter_mut() {
        if let Some(ir) = &mut field.ir {
            ir.modify_io(&mut |io| {
                if let IO::Http { use_batcher, group_by, .. } = io {
                    if has_list_ancestor && group_by.is_some() {
                        field.use_batch_loader = Some(true);
                        *use_batcher = true;
                    }
                }
            });
        }
        mark_direct_loader(
            &mut field.selection,
            field.type_of.is_list() || has_list_ancestor,
        );
    }
}

impl<A> Transform for CheckHttpMerge<A> {
    type Value = OperationPlan<A>;
    type Error = Infallible;

    fn transform(&self, mut plan: Self::Value) -> Valid<Self::Value, Self::Error> {
        mark_direct_loader(&mut plan.selection, false);
        Valid::succeed(plan)
    }
}
