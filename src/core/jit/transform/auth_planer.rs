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
        let mut before = Vec::new();

        plan.selection = plan
            .selection
            .into_iter()
            .map(|field| drop_protect_field(&mut before, field))
            .collect();
        let ir = before.into_iter();

        plan.before = match plan.before {
            Some(before) => Some(ir.fold(before, |a, b| a.pipe(b))),
            None => ir.reduce(|a, b| a.pipe(b)),
        };
        Valid::succeed(plan)
    }
}

/// Used to recursively update the field ands its selections to remove
/// IR::Protected
fn drop_protect_field<A>(before: &mut Vec<IR>, mut field: Field<A>) -> Field<A> {
    if let Some(mut ir) = field.ir {
        let is_protected = drop_protect_ir(&mut ir);

        field.selection = field
            .selection
            .into_iter()
            .map(|field| drop_protect_field(before, field))
            .collect();

        if is_protected {
            let ir = IR::Protect(Box::new(IR::Dynamic(DynamicValue::Value(
                Default::default(),
            ))));
            before.push(ir);
        }

        field.ir = Some(ir);
    }
    field
}

/// This function modifies an IR chain by detecting and removing any
/// instances of IR::Protect from the chain. Returns `true` when it modifies the
/// IR.
pub fn drop_protect_ir(ir: &mut IR) -> bool {
    match ir {
        IR::Dynamic(_) => false,
        IR::IO(_) => false,
        IR::Cache(_) => false,
        IR::Path(ir, _) => drop_protect_ir(ir),
        IR::ContextPath(_) => false,
        IR::Protect(inner_ir) => {
            *ir = *inner_ir.clone();
            true
        }
        IR::Map(_) => false,
        IR::Pipe(ir1, ir2) => drop_protect_ir(ir1) || drop_protect_ir(ir2),
        IR::Merge(_) => false,
        IR::Discriminate(_, ir) => drop_protect_ir(ir),
        IR::Entity(_) => false,
        IR::Service(_) => false,
    }
}
