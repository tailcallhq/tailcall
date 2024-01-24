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
use hyper::body::Bytes;
use mini_v8::{FromValue, MiniV8, ToValue, ToValues, Value as MValue, Values};

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

#[async_trait::async_trait]
pub trait ScriptIO<Event, Command> {
  async fn on_event(&self, event: Event) -> anyhow::Result<Command>;
}

#[derive(Debug)]
pub enum Event {
  Request(reqwest::Request),
  Response(Vec<Response<hyper::body::Bytes>>),
}

#[derive(Debug)]
pub enum Command {
  Request(Vec<reqwest::Request>),
  Response(Response<hyper::body::Bytes>),
}

impl ToValues for Event {
  fn to_values(self, mv8: &MiniV8) -> mini_v8::Result<Values> {
    match self {
      Event::Request(req) => {
        let req = from_req(req, mv8.clone())?;
        let val = Values::from_iter(vec![req]);
        Ok(val)
      }
      Event::Response(responses) => {
        let mut vec = Vec::new();
        for res in responses {
          vec.push(from_resp(res, mv8.clone())?);
        }
        Ok(Values::from_iter(vec))
      }
    }
  }
}

impl ToValue for Response<Bytes> {
  fn to_value(self, mv8: &MiniV8) -> mini_v8::Result<MValue> {
    from_resp(self, mv8.clone())
  }
}

fn from_resp(resp: Response<Bytes>, mv8: MiniV8) -> mini_v8::Result<MValue> {
  let obj = mv8.create_object();
  let status = resp.status.as_u16() as f64;
  let headers = mv8.create_array();
  for (key, value) in resp.headers.iter() {
    let key = mv8.create_string(key.as_str());
    let value = mv8.create_string(value.to_str().unwrap());
    let pair = mv8.create_array();
    pair.set(0, MValue::String(key))?;
    pair.set(1, MValue::String(value))?;
    headers.push(pair)?;
  }
  let body = mv8.create_string(String::from_utf8_lossy(resp.body.as_ref()).as_ref());
  obj.set("status", MValue::Number(status))?;
  obj.set("headers", MValue::Array(headers))?;
  obj.set("body", MValue::String(body))?;
  Ok(MValue::Object(obj))
}

fn from_req(req: reqwest::Request, mv8: MiniV8) -> mini_v8::Result<MValue> {
  let obj = mv8.create_object();
  let url = mv8.create_string(req.url().to_string().as_str());
  let method = mv8.create_string(req.method().clone().as_str());
  let headers = mv8.create_array();
  for (key, value) in req.headers().iter() {
    let key = mv8.create_string(key.as_str());
    let value = mv8.create_string(value.to_str().unwrap());
    let pair = mv8.create_array();
    pair.set(0, MValue::String(key))?;
    pair.set(1, MValue::String(value))?;
    headers.push(pair)?;
  }
  if let Some(body) = req.body() {
    let body = mv8.create_string(String::from_utf8_lossy(body.as_bytes().unwrap()).as_ref());
    obj.set("body", MValue::String(body))?;
  }
  obj.set("url", MValue::String(url))?;
  obj.set("method", MValue::String(method))?;
  Ok(MValue::Object(obj))
}

impl FromValue for Command {
  fn from_value(_value: MValue, _mv8: &MiniV8) -> mini_v8::Result<Self> {
    todo!()
  }
}

trait JSValue: Send + Sync + 'static {
  fn to_values(&self) -> mini_v8::Values;
  fn from_value(value: mini_v8::Value) -> Self;
}

impl JSValue for Event {
  fn to_values(&self) -> mini_v8::Values {
    todo!()
  }

  fn from_value(_: mini_v8::Value) -> Self {
    todo!()
  }
}

impl JSValue for Command {
  fn to_values(&self) -> mini_v8::Values {
    todo!()
  }

  fn from_value(_value: mini_v8::Value) -> Self {
    todo!()
  }
}

impl JSValue for () {
  fn to_values(&self) -> mini_v8::Values {
    mini_v8::Values::new()
  }

  fn from_value(_: mini_v8::Value) -> Self {}
}

impl JSValue for f64 {
  fn to_values(&self) -> mini_v8::Values {
    mini_v8::Values::from_iter(vec![mini_v8::Value::Number(*self)])
  }

  fn from_value(value: mini_v8::Value) -> Self {
    value.as_number().unwrap()
  }
}
