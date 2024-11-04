use std::marker::PhantomData;

use tailcall_valid::Valid;

use crate::core::jit::{Error, Field, OperationPlan, Variables};
use crate::core::json::JsonLike;
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

impl<Var, Value: Clone> Transform for Skip<'_, Var, Value>
where
    Var: for<'b> JsonLike<'b> + Clone,
{
    type Value = OperationPlan<Value>;

    type Error = Error;

    fn transform(&self, plan: Self::Value) -> Valid<Self::Value, Self::Error> {
        let mut plan = plan;
        let variables: &Variables<Var> = self.variables;
        skip(&mut plan.selection, variables);

        Valid::succeed(plan)
    }
}

/// Drops all the fields that are not needed based on the set variables
fn skip<Input, Var: for<'b> JsonLike<'b>>(fields: &mut Vec<Field<Input>>, vars: &Variables<Var>) {
    fields.retain(|f| !f.skip(vars));
    for field in fields {
        skip(&mut field.selection, vars);
    }
}
