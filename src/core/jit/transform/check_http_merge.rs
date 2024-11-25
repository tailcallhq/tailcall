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
                if let IO::Http { dl_enabled, group_by, .. } = io {
                    match (has_list_ancestor, group_by.is_some()) {
                        (true, true) => {
                            // ideal condition
                            // has list ancestor and group by clause
                            field.dl_enabled = Some(true);
                            *dl_enabled = true;
                        }
                        (false, true) => {
                            // has not list ancestor but group by clause
                            field.dl_enabled = Some(true);
                            *dl_enabled = true;
                        }
                        (_, false) => {
                            // has no group by clause means we can't process
                            // it with http merge.
                            *dl_enabled = false;
                            field.dl_enabled = Some(false);
                        }
                    }

                    if has_list_ancestor && group_by.is_some() {
                        field.dl_enabled = Some(true);
                        *dl_enabled = true;
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
