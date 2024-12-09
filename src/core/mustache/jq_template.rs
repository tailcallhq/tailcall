use std::sync::Arc;

use nom::branch::alt;
use nom::bytes::complete::{tag, take_until};
use nom::combinator::map;
use nom::multi::many0;
use nom::sequence::delimited;
use nom::{Finish, IResult};

use super::{JqTransform, Mustache, PathJqValueString};
use crate::core::mustache::{JqTemplateError, Segment};

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum JqTemplateIR {
    JqTransform(JqTransform),
    Literal(String),
    Mustache(Mustache),
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct JqTemplate(pub Vec<JqTemplateIR>);

impl JqTemplate {
    pub fn is_const(&self) -> bool {
        self.0.iter().all(|v| match v {
            JqTemplateIR::JqTransform(jq) => jq.is_const(),
            JqTemplateIR::Literal(_) => true,
            JqTemplateIR::Mustache(m) => m.is_const(),
        })
    }

    // TODO: return error
    pub fn render_value(&self, ctx: &impl PathJqValueString) -> async_graphql_value::ConstValue {
        let expressions_len = self.0.len();
        match expressions_len {
            0 => async_graphql_value::ConstValue::Null,
            1 => {
                let expression = self.0.first().unwrap();
                self.execute_expression(ctx, expression)
            }
            _ => {
                let result = self
                    .0
                    .iter()
                    .map(|expr| self.execute_expression(ctx, expr))
                    .fold(String::new(), |mut acc, cur| {
                        match &cur {
                            async_graphql::Value::String(s) => acc += s,
                            _ => acc += &cur.to_string(),
                        }
                        acc
                    });
                async_graphql_value::ConstValue::String(result)
            }
        }
    }

    fn execute_expression(
        &self,
        ctx: &impl PathJqValueString,
        expression: &JqTemplateIR,
    ) -> async_graphql_value::ConstValue {
        match expression {
            JqTemplateIR::JqTransform(jq_transform) => {
                jq_transform.render_value(super::PathValueEnum::PathValue(Arc::new(ctx)))
            }
            JqTemplateIR::Literal(value) => async_graphql_value::ConstValue::String(value.clone()),
            JqTemplateIR::Mustache(mustache) => {
                let mustache_result = mustache.render(ctx);

                serde_json::from_str::<async_graphql_value::ConstValue>(&mustache_result)
                    .unwrap_or_else(|_| async_graphql_value::ConstValue::String(mustache_result))
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
            // TODO: use the error
            match JqTransform::try_new(template) {
                Ok(jq) => JqTemplateIR::JqTransform(jq),
                Err(err) => match err {
                    JqTemplateError::JqIsMustache => {
                        let expression: Vec<_> = template
                            .trim()
                            .split('.')
                            .skip(1)
                            .map(String::from)
                            .collect();
                        let segment = Segment::Expression(expression);
                        JqTemplateIR::Mustache(Mustache::from(vec![segment]))
                    }
                    _ => JqTemplateIR::Literal(template.to_string()),
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
