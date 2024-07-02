use std::borrow::Cow;

use crate::core::ir::{EvalContext, ResolverContextLike};

#[derive(Default, Debug, Clone)]
pub enum EncodingStrategy {
    CommaSeparated,
    #[default]
    RepeatedKey,
}

pub trait Encoder {
    fn encode<T: AsRef<str>, P: AsRef<str>>(
        &self,
        key: T,
        path: &[P],
        encoding_strategy: &EncodingStrategy,
    ) -> Option<String>;
}

impl EncodingStrategy {
    pub fn encode(&self, key: &str, value: Cow<'_, async_graphql::Value>) -> Option<String> {
        match self {
            Self::CommaSeparated => match &*value {
                async_graphql::Value::List(list) => Some(format!(
                    "{}={}",
                    key,
                    list.iter()
                        .map(|val| val.to_string())
                        .collect::<Vec<String>>()
                        .join(",")
                )),
                _ => encode_value(key, value),
            },
            Self::RepeatedKey => match &*value {
                async_graphql::Value::List(list) => Some(
                    list.iter()
                        .map(|val| format!("{}={}", key, val))
                        .collect::<Vec<String>>()
                        .join("&"),
                ),
                _ => encode_value(key, value),
            },
        }
    }
}

impl<'a, Ctx: ResolverContextLike> Encoder for EvalContext<'a, Ctx> {
    fn encode<T: AsRef<str>, P: AsRef<str>>(
        &self,
        key: T,
        path: &[P],
        encoding_strategy: &EncodingStrategy,
    ) -> Option<String> {
        let ctx = self;

        if path.len() < 2 {
            return None;
        }

        path.split_first()
            .and_then(move |(head, tail)| match head.as_ref() {
                "args" => encoding_strategy.encode(key.as_ref(), ctx.path_arg(tail)?),
                "value" => encoding_strategy.encode(key.as_ref(), ctx.path_value(tail)?),
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

pub fn encode_value(key: &str, value: Cow<'_, async_graphql::Value>) -> Option<String> {
    match value {
        Cow::Owned(async_graphql::Value::String(s)) => Some(format!("{}={}", key, s)),
        Cow::Borrowed(async_graphql::Value::String(s)) => Some(format!("{}={}", key, s)),

        Cow::Owned(async_graphql::Value::Number(n)) => Some(format!("{}={}", key, n)),
        Cow::Borrowed(async_graphql::Value::Number(n)) => Some(format!("{}={}", key, n)),

        Cow::Owned(async_graphql::Value::Boolean(b)) => Some(format!("{}={}", key, b)),
        Cow::Borrowed(async_graphql::Value::Boolean(b)) => Some(format!("{}={}", key, b)),

        _ => None,
    }
}
