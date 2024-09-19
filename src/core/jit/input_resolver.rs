use async_graphql_value::{ConstValue, Value};

use super::{Arg, Field, OperationPlan, ResolveInputError, Variables};
use crate::core::json::{JsonLikeOwned, JsonObjectLike};
use crate::core::Type;

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
    Output: Clone + JsonLikeOwned + TryFrom<serde_json::Value>,
    Input: InputResolvable<Output = Output>,
    <Output as TryFrom<serde_json::Value>>::Error: std::fmt::Debug,
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
                            let value = self.recursive_parse_arg(
                                &field.name,
                                &arg.name,
                                &arg.type_of,
                                &arg.default_value,
                                arg.value,
                            )?;
                            Ok(Arg { value, ..arg })
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

    #[allow(clippy::too_many_arguments)]
    fn recursive_parse_arg(
        &self,
        parent_name: &str,
        arg_name: &str,
        type_of: &Type,
        default_value: &Option<Output>,
        value: Option<Output>,
    ) -> Result<Option<Output>, ResolveInputError> {
        let is_value_null = value.as_ref().map(|val| val.is_null()).unwrap_or(true);
        let value: Result<Option<Output>, ResolveInputError> =
            if !type_of.is_nullable() && value.is_none() {
                let default_value = default_value.clone();
                match default_value {
                    Some(value) => Ok(Some(value)),
                    None => Err(ResolveInputError::ArgumentIsRequired {
                        arg_name: arg_name.to_string(),
                        field_name: parent_name.to_string(),
                    }),
                }
            } else if !type_of.is_nullable() && is_value_null {
                Err(ResolveInputError::ArgumentIsRequired {
                    arg_name: arg_name.to_string(),
                    field_name: parent_name.to_string(),
                })
            } else if value.is_none() {
                let default_value = default_value.clone();
                Ok(default_value)
            } else {
                Ok(value)
            };

        let Some(mut value) = value? else {
            return Ok(None);
        };

        let Some(def) = self.plan.index.get_input_type_definition(type_of.name()) else {
            return Ok(Some(value));
        };

        if let Some(obj) = value.as_object_mut() {
            for arg_field in &def.fields {
                let parent_name = format!("{}.{}", parent_name, arg_name);
                let field_value = obj.get_key(&arg_field.name).cloned();
                let field_default = arg_field
                    .default_value
                    .clone()
                    .map(|value| Output::try_from(value).expect("The conversion cannot fail"));
                let value = self.recursive_parse_arg(
                    &parent_name,
                    &arg_field.name,
                    &arg_field.of_type,
                    &field_default,
                    field_value,
                )?;
                if let Some(value) = value {
                    obj.insert_key(&arg_field.name, value);
                }
            }
        } else if let Some(arr) = value.as_array_mut() {
            for (index, item) in arr.iter_mut().enumerate() {
                let parent_name = format!("{}.{}.{}", parent_name, arg_name, index);

                *item = self
                    .recursive_parse_arg(
                        &parent_name,
                        &index.to_string(),
                        type_of,
                        &None,
                        Some(item.clone()),
                    )?
                    .expect("Because we start with `Some`, we will end with `Some`");
            }
        }

        Ok(Some(value))
    }
}
