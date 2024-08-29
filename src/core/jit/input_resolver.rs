use async_graphql_value::{ConstValue, Value};

use super::{OperationPlan, ResolveInputError, Variables};

/// Trait to represent conversion from some dynamic type (with variables)
/// to the resolved variant based on the additional provided info.
/// E.g. conversion from [async_graphql_value::Value] ->
/// [async_graphql_value::ConstValue]
pub trait InputResolvable {
    type Output;

    fn resolve(
        self,
        variables: &Variables<Self::Output>,
    ) -> Result<Self::Output, ResolveInputError>;
}

impl InputResolvable for Value {
    type Output = ConstValue;

    // TODO:
    // - provide default values
    fn resolve(self, variables: &Variables<ConstValue>) -> Result<Self::Output, ResolveInputError> {
        self.into_const_with(|name| {
            variables
                .get(&name)
                .cloned()
                .ok_or_else(|| ResolveInputError::VariableIsNotFound(name.to_string()))
        })
    }
}

/// Transforms [OperationPlan] values the way that all the input values
/// are transformed to const variant with the help of [InputResolvable] trait
pub struct InputResolver<Input> {
    plan: OperationPlan<Input>,
}

impl<Input> InputResolver<Input> {
    pub fn new(plan: OperationPlan<Input>) -> Self {
        Self { plan }
    }
}

impl<Input, Output> InputResolver<Input>
where
    Input: Clone,
    Output: Clone,
    Input: InputResolvable<Output = Output>,
{
    pub fn resolve_input(
        &self,
        variables: &Variables<Output>,
    ) -> Result<OperationPlan<Output>, ResolveInputError> {
        let new_fields = self
            .plan
            .as_parent()
            .iter()
            .map(|field| field.clone().try_map(|value| value.resolve(variables)))
            .collect::<Result<_, _>>()?;

        Ok(OperationPlan::new(
            new_fields,
            self.plan.operation_type(),
            self.plan.index.clone(),
            self.plan.is_introspection_query(),
        ))
    }
}
