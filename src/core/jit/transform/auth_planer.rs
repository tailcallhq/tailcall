use std::convert::Infallible;

use tailcall_valid::Valid;

use crate::core::blueprint::Auth;
use crate::core::ir::model::IR;
use crate::core::jit::{Field, OperationPlan};
use crate::core::Transform;

pub struct AuthPlaner<A> {
    global_auth_requirements: Option<Auth>,
    marker: std::marker::PhantomData<A>,
}

impl<A> AuthPlaner<A> {
    pub fn new(global_auth_requirements: Option<Auth>) -> Self {
        Self { global_auth_requirements, marker: std::marker::PhantomData }
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
            .map(|field| extract_ir_protect(&mut before, &self.global_auth_requirements, field))
            .collect();

        Valid::succeed(OperationPlan { before, ..plan })
    }
}

/// Used to recursively update the field ands its selections to remove
/// IR::Protected
fn extract_ir_protect<A>(
    before: &mut Vec<IR>,
    global_auth_requirements: &Option<Auth>,
    mut field: Field<A>,
) -> Field<A> {
    if let Some(ir) = field.ir {
        let mut auth_requirements: Vec<Auth> = Vec::new();
        let new_ir =
            detect_and_remove_ir_protect(ir, global_auth_requirements, &mut auth_requirements);

        field.selection = field
            .selection
            .into_iter()
            .map(|selection_field| {
                extract_ir_protect(before, global_auth_requirements, selection_field)
            })
            .collect();

        let auth_requirement = auth_requirements
            .into_iter()
            .reduce(|a, b| a.or(b))
            .map(|a| a.simplify());

        if auth_requirement.is_some() {
            before.push(IR::Protect(
                auth_requirement,
                Box::new(IR::ContextPath(vec!["data".to_string()])),
            ));
        }

        field.ir = Some(new_ir);
    }
    field
}

/// This function modifies an IR pipe chain by detecting and removing any
/// instances of IR::Protect from the chain. Returns `true` when it modifies the
/// IR.
pub fn detect_and_remove_ir_protect(
    ir: IR,
    global_auth_requirements: &Option<Auth>,
    auth_requirements: &mut Vec<Auth>,
) -> IR {
    match ir {
        IR::Dynamic(dynamic_value) => IR::Dynamic(dynamic_value),
        IR::IO(io) => IR::IO(io),
        IR::Cache(cache) => IR::Cache(cache),
        IR::Path(inner_ir, vec) => {
            let new_ir = detect_and_remove_ir_protect(
                *inner_ir,
                global_auth_requirements,
                auth_requirements,
            );
            IR::Path(Box::new(new_ir), vec)
        }
        IR::ContextPath(vec) => IR::ContextPath(vec),
        IR::Protect(requirements, inner_ir) => {
            if let Some(auth) = requirements {
                auth_requirements.push(auth);
            } else if let Some(auth) = global_auth_requirements {
                auth_requirements.push(auth.clone());
            }

            
            detect_and_remove_ir_protect(
                *inner_ir,
                global_auth_requirements,
                auth_requirements,
            )
        }
        IR::Map(map) => IR::Map(map),
        IR::Pipe(ir1, ir2) => {
            let new_ir1 =
                detect_and_remove_ir_protect(*ir1, global_auth_requirements, auth_requirements);
            let new_ir2 =
                detect_and_remove_ir_protect(*ir2, global_auth_requirements, auth_requirements);
            IR::Pipe(Box::new(new_ir1), Box::new(new_ir2))
        }
        IR::Discriminate(discriminator, inner_ir) => {
            let new_ir = detect_and_remove_ir_protect(
                *inner_ir,
                global_auth_requirements,
                auth_requirements,
            );
            IR::Discriminate(discriminator, Box::new(new_ir))
        }
        IR::Entity(hash_map) => IR::Entity(hash_map),
        IR::Service(service) => IR::Service(service),
    }
}
