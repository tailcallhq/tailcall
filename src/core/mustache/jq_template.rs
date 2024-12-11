use std::sync::Arc;

use nom::branch::alt;
use nom::bytes::complete::{tag, take_until};
use nom::combinator::map;
use nom::multi::many0;
use nom::sequence::delimited;
use nom::{Finish, IResult};

use super::{JqRuntimeError, JqTransform, Mustache, PathJqValueString};
use crate::core::mustache::Segment;

#[derive(Debug, Clone, PartialEq, Hash)]
/// Used to represent a mixture of getters mustache, jq transformations and
/// const values templates
pub struct JqTemplate(pub Vec<JqTemplateIR>);

#[derive(Debug, Clone, PartialEq, Hash)]
/// The IR for each part of the template
pub enum JqTemplateIR {
    JqTransform(JqTransform),
    Literal(String),
    Mustache(Mustache),
}

impl JqTemplate {
    /// Used to check if the returned expression resolves to a constant value
    /// always
    pub fn is_const(&self) -> bool {
        self.0.iter().all(|v| match v {
            JqTemplateIR::JqTransform(jq) => jq.is_const(),
            JqTemplateIR::Literal(_) => true,
            JqTemplateIR::Mustache(m) => m.is_const(),
        })
    }

    /// Used to render the template
    pub fn render_value(
        &self,
        ctx: &impl PathJqValueString,
    ) -> Result<async_graphql_value::ConstValue, JqRuntimeError> {
        let expressions_len = self.0.len();
        match expressions_len {
            0 => Ok(async_graphql_value::ConstValue::Null),
            1 => {
                let expression = self.0.first().unwrap();
                self.execute_expression(ctx, expression)
            }
            _ => {
                let (errors, result): (Vec<_>, Vec<_>) = self
                    .0
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
        expression: &JqTemplateIR,
    ) -> Result<async_graphql_value::ConstValue, JqRuntimeError> {
        match expression {
            JqTemplateIR::JqTransform(jq_transform) => {
                jq_transform.render_value(super::PathValueEnum::PathValue(Arc::new(ctx)))
            }
            JqTemplateIR::Literal(value) => {
                Ok(async_graphql_value::ConstValue::String(value.clone()))
            }
            JqTemplateIR::Mustache(mustache) => {
                let mustache_result = mustache.render(ctx);

                Ok(
                    serde_json::from_str::<async_graphql_value::ConstValue>(&mustache_result)
                        .unwrap_or_else(|_| {
                            async_graphql_value::ConstValue::String(mustache_result)
                        }),
                )
            }
        }
    }

    pub fn parse(template: &str) -> Self {
        let result = parse_jq_template(template).finish();
        match result {
            Ok((_, jq_template)) => jq_template,
            Err(_) => Self(vec![JqTemplateIR::Literal(template.to_string())]),
        }
    }
}

fn parse_expression(input: &str) -> IResult<&str, JqTemplateIR> {
    delimited(
        tag("{{"),
        map(take_until("}}"), |template| {
            match JqTransform::try_new(template) {
                Ok(jq) => JqTemplateIR::JqTransform(jq),
                Err(err) => match err {
                    JqRuntimeError::JqIsMustache => {
                        let expression: Vec<_> = template
                            .trim()
                            .trim_start_matches('.')
                            .split(".")
                            .map(String::from)
                            .collect();
                        let segment = Segment::Expression(expression);
                        JqTemplateIR::Mustache(Mustache::from(vec![segment]))
                    }
                    _ => {
                        let m = Mustache::parse(&format!("{{{{{}}}}}", template.trim()));
                        if !m.is_const() {
                            JqTemplateIR::Mustache(m)
                        } else {
                            JqTemplateIR::Literal(template.to_string())
                        }
                    }
                },
            }
        }),
        tag("}}"),
    )(input)
}

fn parse_segment(input: &str) -> IResult<&str, Vec<JqTemplateIR>> {
    let expression_result = many0(alt((
        parse_expression,
        map(take_until("{{"), |txt: &str| {
            JqTemplateIR::Literal(txt.to_string())
        }),
    )))(input);

    if let Ok((remaining, segments)) = expression_result {
        if remaining.is_empty() {
            Ok((remaining, segments))
        } else {
            let mut segments = segments;
            segments.push(JqTemplateIR::Literal(remaining.to_string()));
            Ok(("", segments))
        }
    } else {
        Ok(("", vec![JqTemplateIR::Literal(input.to_string())]))
    }
}

fn parse_jq_template(input: &str) -> IResult<&str, JqTemplate> {
    map(parse_segment, |segments| {
        JqTemplate(
            segments
                .into_iter()
                .filter(|seg| match seg {
                    JqTemplateIR::Literal(s) => (!s.is_empty()) && s != "\"",
                    _ => true,
                })
                .collect(),
        )
    })(input)
}
