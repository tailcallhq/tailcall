use async_graphql_value::{ConstValue, Value};

use super::{Arg, Field, OperationPlan, ResolveInputError, Variables};

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

pub trait OutputTrait {
    fn is_null_value(&self) -> bool;
}

impl OutputTrait for ConstValue {
    fn is_null_value(&self) -> bool {
        self.eq(&ConstValue::Null)
    }
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
    Output: Clone + OutputTrait,
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
            .map(|field| match field {
                Ok(field) => {
                    let args = field
                        .args
                        .into_iter()
                        .map(|arg| {
                            // TODO: this should recursively check the InputType for field presence
                            if arg
                                .value
                                .as_ref()
                                .map(|val| val.is_null_value())
                                .unwrap_or(true)
                                && !arg.type_of.is_nullable()
                            {
                                let default_value = arg.default_value.clone();
                                match default_value {
                                    Some(value) => Ok(Arg { value: Some(value), ..arg }),
                                    None => Err(ResolveInputError::ArgumentIsRequired {
                                        arg_name: arg.name,
                                        field_name: field.output_name.clone(),
                                    }),
                                }
                            } else if arg.value.is_none() {
                                let default_value = arg.default_value.clone();
                                Ok(Arg { value: default_value, ..arg })
                            } else {
                                Ok(arg)
                            }
                        })
                        .collect::<Result<_, _>>()?;

                    Ok(Field { args, ..field })
                }
                Err(err) => Err(err),
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(OperationPlan::new(
            new_fields,
            self.plan.operation_type(),
            self.plan.index.clone(),
            self.plan.is_introspection_query,
        ))
    }
}
