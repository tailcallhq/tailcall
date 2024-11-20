use std::convert::Infallible;

use tailcall_valid::Valid;

use crate::core::blueprint::Auth;
use crate::core::ir::model::IR;
use crate::core::jit::OperationPlan;
use crate::core::Transform;

pub struct AuthPlaner<'a, A> {
    marker: std::marker::PhantomData<A>,
    auth: &'a Option<Auth>,
}

impl<'a, A> AuthPlaner<'a, A> {
    pub fn new(auth: &'a Option<Auth>) -> Self {
        Self { marker: std::marker::PhantomData, auth }
    }
}

impl<A> Transform for AuthPlaner<'_, A> {
    type Value = OperationPlan<A>;
    type Error = Infallible;

    fn transform(&self, plan: Self::Value) -> Valid<Self::Value, Self::Error> {
        let is_protected = plan.iter_dfs().all(|field| match field.ir {
            Some(ref ir) => is_protected(ir),
            None => true,
        });

        if is_protected {
            Valid::succeed(OperationPlan { auth_n: self.auth.clone(), ..plan })
        } else {
            Valid::succeed(plan)
        }
    }
}

/// Checks if the IR will always evaluate to a Protected Value
pub fn is_protected(ir: &IR) -> bool {
    match ir {
        IR::Dynamic(_) => false,
        IR::IO(_) => false,
        IR::Cache(_) => false,
        IR::Path(ir, _) => is_protected(ir),
        IR::ContextPath(_) => false,
        IR::Protect(_) => true,
        IR::Map(map) => is_protected(&map.input),
        IR::Pipe(ir, ir1) => is_protected(ir) || is_protected(ir1),
        IR::Discriminate(_, ir) => is_protected(ir),
        IR::Entity(hash_map) => hash_map.values().any(is_protected),
        IR::Service(_) => false,
    }
}
