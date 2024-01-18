#![allow(clippy::module_inception)]
#![allow(clippy::mutable_key_type)]
mod app_context;
pub mod async_graphql_hyper;
pub mod blueprint;
pub mod cache;
pub mod chrono_cache;
#[cfg(feature = "default")]
pub mod cli;
pub mod config;
pub mod data_loader;
pub mod directive;
pub mod document;
pub mod endpoint;
pub mod graphql;
pub mod grpc;
pub mod has_headers;
pub mod helpers;
pub mod http;
#[cfg(feature = "unsafe-js")]
pub mod javascript;
pub mod json;
pub mod lambda;
pub mod mustache;
pub mod path;
pub mod print_schema;
pub mod try_fold;
pub mod valid;

use std::future::Future;
use std::hash::Hash;
use std::num::NonZeroU64;

use http::Response;

pub trait EnvIO: Send + Sync + 'static {
  fn get(&self, key: &str) -> Option<String>;
}

#[async_trait::async_trait]
pub trait HttpIO: Sync + Send + 'static {
  async fn execute(&self, request: reqwest::Request) -> anyhow::Result<Response<hyper::body::Bytes>>;
}

pub trait FileIO {
  fn write<'a>(&'a self, file: &'a str, content: &'a [u8]) -> impl Future<Output = anyhow::Result<()>>;
  fn read<'a>(&'a self, file_path: &'a str) -> impl Future<Output = anyhow::Result<String>>;
}

#[async_trait::async_trait]
pub trait Cache<K: Hash + Eq, V>: Send + Sync {
  #[allow(clippy::too_many_arguments)]
  async fn insert<'a>(&'a self, key: K, value: V, ttl: NonZeroU64) -> anyhow::Result<V>;
  async fn get<'a>(&'a self, key: &'a K) -> anyhow::Result<V>;
}
