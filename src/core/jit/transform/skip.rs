use crate::core::{
    jit::{Error, OperationPlan, Variables},
    json::JsonLike,
    valid::Valid,
    Transform,
};

pub struct Skip<A> {
    variables: Variables<A>,
}

impl<A> Skip<A> {
    pub fn new(variables: Variables<A>) -> Self {
        Self { variables }
    }
}

impl<A> Transform for Skip<A>
where
    A: for<'a> JsonLike<'a> + Clone,
{
    type Value = OperationPlan<A>;

    type Error = Error;

    fn transform(&self, mut value: Self::Value) -> Valid<Self::Value, Self::Error> {
        // Drop all the fields that are not needed
        value.flat.retain(|f| !f.skip(&self.variables));

        // Recreate a plan with the new fields
        let plan = OperationPlan::new(
            &value.root_name,
            value.flat,
            value.operation_type,
            value.index,
            value.is_introspection_query,
        );

        Valid::succeed(plan)
    }
}
