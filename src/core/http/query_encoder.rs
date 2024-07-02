use std::borrow::Cow;

use crate::core::ir::{EvalContext, ResolverContextLike};
use crate::core::mustache::Segment;
use crate::core::Mustache;

// Only knows and cares about how to encode query args values.
pub struct QueryEncoder<'a, Ctx: ResolverContextLike> {
    ctx: &'a EvalContext<'a, Ctx>,
}

pub fn encode_value(key: &str, value: Cow<'_, async_graphql::Value>) -> Option<String> {
    match value {
        Cow::Owned(async_graphql::Value::String(s)) => Some(format!("{}={}", key, s)),
        Cow::Borrowed(async_graphql::Value::String(s)) => Some(format!("{}={}", key, s)),

        Cow::Owned(async_graphql::Value::Number(n)) => Some(format!("{}={}", key, n)),
        Cow::Borrowed(async_graphql::Value::Number(n)) => Some(format!("{}={}", key, n)),

        Cow::Owned(async_graphql::Value::Boolean(b)) => Some(format!("{}={}", key, b)),
        Cow::Borrowed(async_graphql::Value::Boolean(b)) => Some(format!("{}={}", key, b)),

        Cow::Owned(async_graphql::Value::List(list)) => Some(
            list.iter()
                .map(|val| format!("{}={}", key, val))
                .fold("".to_string(), |str, item| {
                    if str.is_empty() {
                        item
                    } else if item.is_empty() {
                        str
                    } else {
                        format!("{}&{}", str, item)
                    }
                }),
        ),
        Cow::Borrowed(async_graphql::Value::List(list)) => Some(
            list.iter()
                .map(|val| format!("{}={}", key, val))
                .fold("".to_string(), |str, item| {
                    if str.is_empty() {
                        item
                    } else if item.is_empty() {
                        str
                    } else {
                        format!("{}&{}", str, item)
                    }
                }),
        ),
        _ => None,
    }
}

impl<'a, Ctx: ResolverContextLike> QueryEncoder<'a, Ctx> {
    pub fn new(ctx: &'a EvalContext<'a, Ctx>) -> Self {
        Self { ctx }
    }

    pub fn encode_expr<T: AsRef<str>, P: AsRef<str>>(&self, key: P, path: &[T]) -> Option<String> {
        let ctx = self.ctx;

        if path.len() < 2 {
            return None;
        }

        path.split_first()
            .and_then(move |(head, tail)| match head.as_ref() {
                "args" => encode_value(key.as_ref(), ctx.path_arg(tail)?),
                "value" => encode_value(key.as_ref(), ctx.path_value(tail)?),
                "vars" => ctx
                    .var(tail[0].as_ref())
                    .map(|v| format!("{}={}", key.as_ref(), v)),
                "env" => ctx
                    .env_var(tail[0].as_ref())
                    .map(|v| format!("{}={}", key.as_ref(), v)),
                _ => None,
            })
    }
}

pub trait Encoder {
    fn encode<T: AsRef<str>>(&self, key: T, mustache: &Mustache) -> String;
}

impl<'a, Ctx: ResolverContextLike> Encoder for EvalContext<'a, Ctx> {
    fn encode<T: AsRef<str>>(&self, key: T, mustache: &Mustache) -> String {
        let ctx = self;
        mustache
            .segments()
            .iter()
            .map(|segment| match segment {
                Segment::Literal(text) => format!("{}={}", key.as_ref(), text),
                Segment::Expression(parts) => QueryEncoder::new(ctx)
                    .encode_expr(key.as_ref(), parts)
                    .map(|a| a.to_string())
                    .unwrap_or_default(),
            })
            .collect()
    }
}
