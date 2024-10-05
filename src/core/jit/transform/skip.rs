use std::marker::PhantomData;

use crate::core::jit::{Error, OperationPlan, Variables};
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

    type Error = Error;

    fn transform(&self, mut plan: Self::Value) -> Valid<Self::Value, Self::Error> {
        // Drop all the fields that are not needed
        plan.flat.retain(|f| !f.skip(self.variables));

        

        // Recreate a plan with the new fields
        let plan = OperationPlan::new(
            &plan.root_name,
            plan.flat,
            plan.operation_type,
            plan.index,
            plan.is_introspection_query,
        );

        Valid::succeed(plan)
    }
}
