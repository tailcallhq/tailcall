use std::convert::Infallible;

use tailcall_valid::Valid;

use crate::core::ir::model::{IO, IR};
use crate::core::jit::{Field, OperationPlan};
use crate::core::Transform;

pub struct CheckDirectLoader<A>(std::marker::PhantomData<A>);
impl<A> CheckDirectLoader<A> {
    pub fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}

fn can_ir_use_batch_loader(ir: &mut IR, is_parent_list: bool) -> bool {
    let mut use_batch_loader = false;
    match ir {
        IR::IO(IO::Http { bl_id, .. }) => {
            if is_parent_list && bl_id.is_some() {
                use_batch_loader = true;
            }
            if !use_batch_loader {
                *bl_id = None;
            }
        }
        IR::Cache(cache) => {
            let io: &mut IO = &mut cache.io;
            if let IO::Http { bl_id, .. } = io {
                if is_parent_list && bl_id.is_some() {
                    use_batch_loader = true;
                }
                if !use_batch_loader {
                    *bl_id = None;
                }
            }
        }
        IR::Discriminate(_, ir) | IR::Protect(ir) | IR::Path(ir, _) => {
            use_batch_loader = can_ir_use_batch_loader(ir, is_parent_list);
        }
        IR::Pipe(ir1, ir2) => {
            use_batch_loader = can_ir_use_batch_loader(ir1, is_parent_list)
                && can_ir_use_batch_loader(ir2, is_parent_list);
        }
        IR::Entity(hash_map) => {
            use_batch_loader = hash_map
                .values_mut()
                .all(|ir| can_ir_use_batch_loader(ir, is_parent_list));
        }
        IR::Map(map) => {
            use_batch_loader = can_ir_use_batch_loader(&mut map.input, is_parent_list);
        }
        _ => {}
    }
    use_batch_loader
}

fn mark_direct_loader<A>(selection: &mut [Field<A>], is_parent_list: bool) {
    for field in selection.iter_mut() {
        if let Some(ir) = &mut field.ir {
            if can_ir_use_batch_loader(ir, is_parent_list) {
                field.use_batch_loader = Some(true);
            } else {
                field.use_batch_loader = Some(false);
            }
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
