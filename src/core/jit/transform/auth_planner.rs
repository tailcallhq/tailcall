use std::convert::Infallible;
use std::fmt::Debug;

use tailcall_valid::Valid;

use crate::core::blueprint::{Auth, DynamicValue};
use crate::core::ir::model::IR;
use crate::core::jit::{Field, OperationPlan};
use crate::core::Transform;

pub struct AuthPlanner<A> {
    marker: std::marker::PhantomData<A>,
}

impl<A> AuthPlanner<A> {
    pub fn new() -> Self {
        Self { marker: std::marker::PhantomData }
    }
}

impl<A: Debug> Transform for AuthPlanner<A> {
    type Value = OperationPlan<A>;
    type Error = Infallible;

    fn transform(&self, mut plan: Self::Value) -> Valid<Self::Value, Self::Error> {
        let mut auth = Vec::new();
        plan.selection
            .iter_mut()
            .for_each(|field| update_field(&mut auth, field));

        plan.before = auth
            .into_iter()
            .reduce(|a, b| a.and(b))
            .map(|auth| IR::Protect(auth, Box::new(IR::Dynamic(DynamicValue::default()))));

        Valid::succeed(plan)
    }
}

/// Used to recursively update the field ands its selections to remove
/// IR::Protected
fn update_field<A>(auth: &mut Vec<Auth>, field: &mut Field<A>) {
    if let Some(ref mut ir) = field.ir {
        update_ir(ir, auth);

        field
            .selection
            .iter_mut()
            .for_each(|field| update_field(auth, field));
    }
}

/// This function modifies an IR pipe chain by detecting and removing any
/// instances of IR::Protect from the chain. Returns `true` when it modifies the
/// IR.
pub fn update_ir(ir: &mut IR, vec: &mut Vec<Auth>) {
    match ir {
        IR::Dynamic(_)
        | IR::IO(_)
        | IR::Cache(_)
        | IR::ContextPath(_)
        | IR::Map(_)
        | IR::Entity(_)
        | IR::Service(_) => {}
        IR::Path(ir, _) => {
            update_ir(ir, vec);
        }
        IR::Protect(auth, ir_0) => {
            vec.push(auth.clone());

            update_ir(ir_0, vec);
            *ir = *ir_0.clone();
        }
        IR::Pipe(ir1, ir2) => {
            update_ir(ir1, vec);
            update_ir(ir2, vec);
        }
        IR::Discriminate(_, ir) => {
            update_ir(ir, vec);
        }
        IR::Merge(irs) => {
            irs.iter_mut().for_each(|ir| update_ir(ir, vec));
        }
    }
}
