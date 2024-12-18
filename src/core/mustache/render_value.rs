use std::sync::Arc;

use super::{JqRuntimeError, Mustache, PathJqValueString, Segment};

impl Mustache {
    /// Used to render the template
    pub fn render_value(
        &self,
        ctx: &impl PathJqValueString,
    ) -> Result<async_graphql_value::ConstValue, JqRuntimeError> {
        let expressions_len = self.segments().len();
        match expressions_len {
            0 => Ok(async_graphql_value::ConstValue::Null),
            1 => {
                let expression = self.segments().first().unwrap();
                self.execute_expression(ctx, expression)
            }
            _ => {
                let (errors, result): (Vec<_>, Vec<_>) = self
                    .segments()
                    .iter()
                    .map(|expr| self.execute_expression(ctx, expr))
                    .partition(Result::is_err);

                let errors: Vec<JqRuntimeError> = errors
                    .into_iter()
                    .filter_map(|e| match e {
                        Ok(_) => None,
                        Err(err) => Some(err),
                    })
                    .collect();
                if !errors.is_empty() {
                    return Err(JqRuntimeError::JqRuntimeErrors(errors));
                }

                let result = result
                    .into_iter()
                    .filter_map(|v| match v {
                        Ok(v) => Some(v),
                        Err(_) => None,
                    })
                    .fold(String::new(), |mut acc, cur| {
                        match &cur {
                            async_graphql::Value::String(s) => acc += s,
                            _ => acc += &cur.to_string(),
                        }
                        acc
                    });
                Ok(async_graphql_value::ConstValue::String(result))
            }
        }
    }

    fn execute_expression(
        &self,
        ctx: &impl PathJqValueString,
        expression: &Segment,
    ) -> Result<async_graphql_value::ConstValue, JqRuntimeError> {
        match expression {
            Segment::JqTransform(jq_transform) => {
                jq_transform.render_value(super::PathValueEnum::PathValue(Arc::new(ctx)))
            }
            Segment::Literal(value) => Ok(async_graphql_value::ConstValue::String(value.clone())),
            Segment::Expression(parts) => {
                let mustache_result = ctx
                    .path_string(parts)
                    .map(|a| a.to_string())
                    .unwrap_or_default();

                Ok(
                    serde_json::from_str::<async_graphql_value::ConstValue>(&mustache_result)
                        .unwrap_or_else(|_| {
                            async_graphql_value::ConstValue::String(mustache_result)
                        }),
                )
            }
        }
    }
}
