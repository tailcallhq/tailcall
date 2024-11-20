use std::convert::Infallible;

use tailcall_valid::Valid;

use crate::core::blueprint::Auth;
use crate::core::ir::model::IR;
use crate::core::jit::OperationPlan;
use crate::core::Transform;

pub struct AuthPlaner<'a, A> {
    marker: std::marker::PhantomData<A>,
    auth: &'a Option<Auth>
}

impl<'a, A> AuthPlaner<'a, A> {
    pub fn new(auth: &'a Option<Auth>) -> Self {
        Self {
            marker: std::marker::PhantomData,
            auth
        }
    }
}

impl<'a, A> Transform for AuthPlaner<'a, A> {
    type Value = OperationPlan<A>;
    type Error = Infallible;

    fn transform(&self, plan: Self::Value) -> Valid<Self::Value, Self::Error> {
        for field in plan.iter_dfs() {
            if let Some(ir) = field.ir.as_ref() {
                match ir {
                    IR::Protect(_) => {
                        return Valid::succeed(OperationPlan {
                            root_name: plan.root_name,
                            operation_type: plan.operation_type,
                            index: plan.index,
                            is_introspection_query: plan.is_introspection_query,
                            is_dedupe: plan.is_dedupe,
                            is_const: plan.is_const,
                            is_protected: true,
                            min_cache_ttl: plan.min_cache_ttl,
                            selection: plan.selection,
                            auth_n: self.auth.clone(),
                        });
                    },
                    _ => {}
                }
            }
        }

        Valid::succeed(plan)
    }
}
