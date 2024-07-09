use async_graphql_value::{ConstValue, Value};

use super::{ExecutionPlan, ResolveInputError};

pub trait InputResolvable {
    type Output;

    fn resolve(self) -> Result<Self::Output, ResolveInputError>;
}

impl InputResolvable for Value {
    type Output = ConstValue;

    // TODO:
    // - implement resolve based on variables
    // - provide default values
    fn resolve(self) -> Result<Self::Output, ResolveInputError> {
        self.into_const().ok_or(ResolveInputError)
    }
}

pub struct InputResolver<Input: Clone> {
    plan: ExecutionPlan<Input>,
}

impl<Input: Clone> InputResolver<Input> {
    pub fn new(plan: ExecutionPlan<Input>) -> Self {
        Self { plan }
    }
}

impl<Input: Clone, Output> InputResolver<Input>
where
    Output: Clone,
    Input: InputResolvable<Output = Output>,
{
    pub fn resolve_input(&self) -> Result<ExecutionPlan<Output>, ResolveInputError> {
        let new_fields = self
            .plan
            .as_parent()
            .iter()
            .map(|field| {
                field
                    .clone()
                    .map_args(|arg| arg.map_value(|value| value.resolve()))
            })
            .collect::<Result<_, _>>()?;

        Ok(ExecutionPlan::new(new_fields))
    }
}
