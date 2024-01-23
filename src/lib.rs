#![allow(clippy::module_inception)]
#![allow(clippy::mutable_key_type)]
mod app_context;
pub mod async_graphql_hyper;
pub mod blueprint;
pub mod cache;
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

use async_graphql_value::ConstValue;
use http::Response;
use mini_v8::{FromValue, ToValues};

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
pub trait Cache: Send + Sync {
  type Key: Hash + Eq;
  type Value;
  async fn set<'a>(&'a self, key: Self::Key, value: Self::Value, ttl: NonZeroU64) -> anyhow::Result<Self::Value>;
  async fn get<'a>(&'a self, key: &'a Self::Key) -> anyhow::Result<Self::Value>;
}

pub type EntityCache = dyn Cache<Key = u64, Value = ConstValue>;

pub trait ScriptIO<Event, Command> {
  fn event_handler(&self) -> anyhow::Result<impl EventHandler<Event, Command>>;
}

pub trait EventHandler<Event, Command>: Send + Sync {
  fn on_event(&self, event: Event) -> anyhow::Result<Command>;
}

pub enum Event {
  Request(reqwest::Request),
  Response(Vec<Response<hyper::body::Bytes>>),
}

pub enum Command {
  Request(Vec<reqwest::Request>),
  Response(Response<hyper::body::Bytes>),
}

impl ToValues for Event {
  fn to_values(self, _mv8: &mini_v8::MiniV8) -> mini_v8::Result<mini_v8::Values> {
    todo!()
  }
}

impl FromValue for Command {
  fn from_value(_value: mini_v8::Value, _mv8: &mini_v8::MiniV8) -> mini_v8::Result<Self> {
    todo!()
  }
}
