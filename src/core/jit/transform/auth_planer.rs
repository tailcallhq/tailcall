use std::convert::Infallible;

use tailcall_valid::Valid;

use crate::core::blueprint::DynamicValue;
use crate::core::ir::model::IR;
use crate::core::jit::{Field, OperationPlan};
use crate::core::Transform;

pub struct AuthPlaner<A> {
    marker: std::marker::PhantomData<A>,
}

impl<A> AuthPlaner<A> {
    pub fn new() -> Self {
        Self { marker: std::marker::PhantomData }
    }
}

impl<A> Transform for AuthPlaner<A> {
    type Value = OperationPlan<A>;
    type Error = Infallible;

    fn transform(&self, mut plan: Self::Value) -> Valid<Self::Value, Self::Error> {
        let mut before = plan.before;

        plan.selection = plan
            .selection
            .into_iter()
            .map(|field| extract_ir_protect(&mut before, field))
            .collect();

        Valid::succeed(OperationPlan { before, ..plan })
    }
}

/// Used to recursively update the field ands its selections to remove
/// IR::Protected
fn extract_ir_protect<A>(before: &mut Option<IR>, mut field: Field<A>) -> Field<A> {
    if let Some(ir) = field.ir {
        let (new_ir, is_protected) = detect_and_remove_ir_protect(ir);

        field.selection = field
            .selection
            .into_iter()
            .map(|selection_field| extract_ir_protect(before, selection_field))
            .collect();

        if is_protected {
            let ir = IR::Protect(Box::new(IR::Dynamic(DynamicValue::Value(Default::default()))));
            *before = Some(ir);
        }

        field.ir = Some(new_ir);
    }
    field
}

/// This function modifies an IR pipe chain by detecting and removing any
/// instances of IR::Protect from the chain. Returns `true` when it modifies the
/// IR.
pub fn detect_and_remove_ir_protect(ir: IR) -> (IR, bool) {
    match ir {
        IR::Dynamic(dynamic_value) => (IR::Dynamic(dynamic_value), false),
        IR::IO(io) => (IR::IO(io), false),
        IR::Cache(cache) => (IR::Cache(cache), false),
        IR::Path(inner_ir, vec) => {
            let (new_ir, removed) = detect_and_remove_ir_protect(*inner_ir);
            (IR::Path(Box::new(new_ir), vec), removed)
        }
        IR::ContextPath(vec) => (IR::ContextPath(vec), false),
        IR::Protect(inner_ir) => {
            let (new_ir, _) = detect_and_remove_ir_protect(*inner_ir);
            (new_ir, true)
        }
        IR::Map(map) => (IR::Map(map), false),
        IR::Pipe(ir1, ir2) => {
            let (new_ir1, removed1) = detect_and_remove_ir_protect(*ir1);
            let (new_ir2, removed2) = detect_and_remove_ir_protect(*ir2);
            (
                IR::Pipe(Box::new(new_ir1), Box::new(new_ir2)),
                removed1 || removed2,
            )
        }
        IR::Discriminate(discriminator, inner_ir) => {
            let (new_ir, removed) = detect_and_remove_ir_protect(*inner_ir);
            (IR::Discriminate(discriminator, Box::new(new_ir)), removed)
        }
        IR::Entity(hash_map) => (IR::Entity(hash_map), false),
        IR::Service(service) => (IR::Service(service), false),
    }
}
