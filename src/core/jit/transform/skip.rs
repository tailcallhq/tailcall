use std::marker::PhantomData;

use crate::core::jit::{Error, Field, OperationPlan, Variables};
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

    fn transform(&self, plan: Self::Value) -> Valid<Self::Value, Self::Error> {
        // Drop all the fields that are not needed
        let plan = filter_skipped(plan, self.variables);

        Valid::succeed(plan)
    }
}

/// Remove fields which are skipped
pub fn filter_skipped<Input, Var: for<'b> JsonLike<'b>>(
    mut op: OperationPlan<Input>,
    variables: &Variables<Var>,
) -> OperationPlan<Input> {
    filter_skipped_fields(&mut op.selection, variables);

    op
}

// TODO: review and rename
fn filter_skipped_fields<Input, Var: for<'b> JsonLike<'b>>(
    fields: &mut Vec<Field<Input>>,
    vars: &Variables<Var>,
) {
    fields.retain(|f| !f.skip(vars));
    for field in fields {
        filter_skipped_fields(&mut field.selection, vars);
    }
}
