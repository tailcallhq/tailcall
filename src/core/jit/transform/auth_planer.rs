use std::convert::Infallible;

use tailcall_valid::Valid;

use crate::core::blueprint::Auth;
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
        if plan.is_protected {
            Valid::succeed(OperationPlan { auth_n: self.auth.clone(), ..plan })
        } else {
            Valid::succeed(plan)
        }
    }
}
