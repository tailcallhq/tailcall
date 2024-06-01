#![allow(clippy::module_inception)]
#![allow(clippy::mutable_key_type)]

mod app_context;
pub mod async_cache;
pub mod async_graphql_hyper;
mod auth;
pub mod blueprint;
pub mod cache;
pub mod config;
pub mod data_loader;
pub mod directive;
pub mod document;
pub mod endpoint;
pub mod generator;
pub mod graphql;
pub mod grpc;
pub mod has_headers;
pub mod helpers;
pub mod http;
pub mod ir;
pub mod json;
pub mod merge_right;
pub mod mustache;
pub mod path;
pub mod primitive;
pub mod print_schema;
mod proto_reader;
mod resource_reader;
pub mod rest;
pub mod runtime;
pub mod scalar;
pub mod schema_extension;
mod serde_value_ext;
pub mod tracing;
pub mod try_fold;
pub mod valid;
pub mod worker;

// Re-export everything from `tailcall_macros` as `macros`
use std::borrow::Cow;
use std::hash::Hash;
use std::num::NonZeroU64;
use std::str::FromStr;
use async_graphql_value::Name;

use http::Response;
pub use tailcall_macros as macros;

pub type BorrowedValue = serde_json_borrow::Value<'static>;

pub trait FromValue {
    fn from_value(value: serde_json_borrow::Value) -> Self;
    fn into_bvalue(self) -> BorrowedValue;
}

impl FromValue for async_graphql_value::ConstValue {
    fn from_value(value: serde_json_borrow::Value) -> Self {
        match value {
            serde_json_borrow::Value::Null => async_graphql_value::ConstValue::Null,
            serde_json_borrow::Value::Bool(b) => async_graphql_value::ConstValue::Boolean(b),
            serde_json_borrow::Value::Number(n) => async_graphql_value::ConstValue::Number(n.into()),
            serde_json_borrow::Value::Str(s) => async_graphql_value::ConstValue::String(s.into()),
            serde_json_borrow::Value::Array(a) => {
                async_graphql_value::ConstValue::List(a.into_iter().map(|v| Self::from_value(v)).collect())
            }
            serde_json_borrow::Value::Object(o) => async_graphql_value::ConstValue::Object(
                o.into_vec().into_iter()
                    .map(|(k, v)| (Name::new(k), Self::from_value(v)))
                    .collect(),
            ),
        }
    }

    fn into_bvalue(self) -> BorrowedValue {
        match self {
            async_graphql_value::ConstValue::Null => serde_json_borrow::Value::Null,
            async_graphql_value::ConstValue::Boolean(b) => serde_json_borrow::Value::Bool(b),
            async_graphql_value::ConstValue::Number(n) => serde_json_borrow::Value::Number(serde_json_borrow::Number::from_str(&n.to_string()).unwrap()), // TODO: FIXME
            async_graphql_value::ConstValue::String(s) => serde_json_borrow::Value::Str(Cow::Owned(s)),
            async_graphql_value::ConstValue::List(a) => serde_json_borrow::Value::Array(a.into_iter().map(|v| v.into_bvalue()).collect::<Vec<_>>().into()),
            async_graphql_value::ConstValue::Object(o) => serde_json_borrow::Value::Object(
                o.into_iter()
                    .map(|(k, v)| (k.to_string(), v.into_bvalue()))
                    .collect::<Vec<_>>().into(),
            ),
            async_graphql_value::ConstValue::Binary(_) => todo!(),
            async_graphql_value::ConstValue::Enum(_) => todo!(),
        }
    }
}

pub fn extend_lifetime<'b>(r: serde_json_borrow::Value<'b>) -> serde_json_borrow::Value<'static> {
    unsafe { std::mem::transmute::<serde_json_borrow::Value<'b>, serde_json_borrow::Value<'static>>(r) }
}

pub fn extend_lifetime_ref<'b>(r: &serde_json_borrow::Value<'b>) -> &'static serde_json_borrow::Value<'static> {
    unsafe { std::mem::transmute::<&serde_json_borrow::Value<'b>, &'static serde_json_borrow::Value<'static>>(r) }
}

pub type ConstValueDe<'de> = serde_json_borrow::Value<'de>;

pub trait IntoConst {
    fn into_const(self) -> async_graphql_value::ConstValue;
}

impl IntoConst for BorrowedValue {
    fn into_const(self) -> async_graphql_value::ConstValue {
        match self {
            serde_json_borrow::Value::Null => async_graphql_value::ConstValue::Null,
            serde_json_borrow::Value::Bool(b) => async_graphql_value::ConstValue::Boolean(b),
            serde_json_borrow::Value::Number(n) => async_graphql_value::ConstValue::Number(n.into()),
            serde_json_borrow::Value::Str(s) => async_graphql_value::ConstValue::String(s.into()),
            serde_json_borrow::Value::Array(a) => {
                async_graphql_value::ConstValue::List(a.into_iter().map(|v| v.into_const()).collect())
            }
            serde_json_borrow::Value::Object(o) => async_graphql_value::ConstValue::Object(
                o.into_vec()
                    .into_iter()
                    .map(|(k, v)| (Name::new(k), v.into_const()))
                    .collect(),
            ),
        }
    }
}


pub trait EnvIO: Send + Sync + 'static {
    fn get(&self, key: &str) -> Option<Cow<'_, str>>;
}

#[async_trait::async_trait]
pub trait HttpIO: Sync + Send + 'static {
    async fn execute(
        &self,
        request: reqwest::Request,
    ) -> anyhow::Result<Response<hyper::body::Bytes>>;
}

#[async_trait::async_trait]
pub trait FileIO: Send + Sync {
    async fn write<'a>(&'a self, path: &'a str, content: &'a [u8]) -> anyhow::Result<()>;
    async fn read<'a>(&'a self, path: &'a str) -> anyhow::Result<String>;
}

#[async_trait::async_trait]
pub trait Cache: Send + Sync {
    type Key: Hash + Eq;
    type Value;
    async fn set<'a>(
        &'a self,
        key: Self::Key,
        value: Self::Value,
        ttl: NonZeroU64,
    ) -> anyhow::Result<()>;
    async fn get<'a>(&'a self, key: &'a Self::Key) -> anyhow::Result<Option<Self::Value>>;

    fn hit_rate(&self) -> Option<f64>;
}

pub type EntityCache = dyn Cache<Key = u64, Value =BorrowedValue>;

#[async_trait::async_trait]
pub trait WorkerIO<In, Out>: Send + Sync + 'static {
    /// Calls a global JS function
    async fn call(&self, name: &'async_trait str, input: In) -> anyhow::Result<Option<Out>>;
}

pub fn is_default<T: Default + Eq>(val: &T) -> bool {
    *val == T::default()
}

#[cfg(test)]
pub mod tests {
    use std::collections::HashMap;

    use super::*;

    #[derive(Clone, Default)]
    pub struct TestEnvIO(HashMap<String, String>);

    impl EnvIO for TestEnvIO {
        fn get(&self, key: &str) -> Option<Cow<'_, str>> {
            self.0.get(key).map(Cow::from)
        }
    }

    impl FromIterator<(String, String)> for TestEnvIO {
        fn from_iter<T: IntoIterator<Item = (String, String)>>(iter: T) -> Self {
            Self(HashMap::from_iter(iter))
        }
    }
}
