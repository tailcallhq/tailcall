use std::{convert::Infallible, marker::PhantomData};

use crate::core::jit::{OperationPlan, Variables};
use crate::core::json::JsonLike;
use crate::core::valid::Valid;
use crate::core::Transform;

pub struct Skip<'a, Var, Value> {
    variables: &'a Variables<Var>,
    _value: PhantomData<Value>,
}

impl<'a, Var, Value> Skip<'a, Var, Value> {
    pub fn new(variables: &'a Variables<Var>) -> Self {
        Self { variables, _value: PhantomData }
    }
}

impl<'a, Var, Value: Clone> Transform for Skip<'a, Var, Value>
where
    Var: for<'b> JsonLike<'b> + Clone,
{
    type Value = OperationPlan<Value>;

    type Error = Infallible;

    fn transform(&self, plan: Self::Value) -> Valid<Self::Value, Self::Error> {
        // Drop all the fields that are not needed
        let plan = plan.filter_skipped(self.variables);

        // Recreate a plan with the new fields
        let plan = OperationPlan::new(
            &plan.root_name,
            plan.selection,
            plan.operation_type,
            plan.index,
            plan.is_introspection_query,
        );

        Valid::succeed(plan)
    }
}
