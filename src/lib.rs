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

use std::collections::BTreeMap;
use std::hash::Hash;
use std::num::NonZeroU64;

use async_graphql_value::ConstValue;
use http::Response;
use hyper::body::Bytes;
use mini_v8::{FromValue, MiniV8, ToValue, ToValues, Value as MValue, Value, Values};
use reqwest::header::{HeaderName, HeaderValue};
use reqwest::Body;

pub trait EnvIO: Send + Sync + 'static {
  fn get(&self, key: &str) -> Option<String>;
}

#[async_trait::async_trait]
pub trait HttpIO: Sync + Send + 'static {
  async fn execute(&self, request: reqwest::Request) -> anyhow::Result<Response<hyper::body::Bytes>>;
}

#[async_trait::async_trait]
pub trait FileIO {
  async fn write<'a>(&'a self, path: &'a str, content: &'a [u8]) -> anyhow::Result<()>;
  async fn read<'a>(&'a self, path: &'a str) -> anyhow::Result<String>;
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
pub trait ScriptIO<Event, Command>: Send + Sync {
  async fn on_event(&self, event: Event) -> anyhow::Result<Command>;
}

#[derive(Debug)]
pub enum Event {
  Request(reqwest::Request),
  Response(Vec<Response<Bytes>>),
}

impl PartialEq for Event {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (Event::Request(req1), Event::Request(req2)) => {
        req1.url() == req2.url()
          && req1.method() == req2.method()
          && req1.headers() == req2.headers()
          && req1.body().unwrap_or(&Body::from("".as_bytes())).as_bytes()
            == req2.body().unwrap_or(&Body::from("".as_bytes())).as_bytes()
      }
      (Event::Response(res1), Event::Response(res2)) => res1.iter().zip(res2.iter()).all(|(r1, r2)| {
        let r1 = r1.clone().to_resp_string().unwrap();
        let r2 = r2.clone().to_resp_string().unwrap();
        r1.status == r2.status && r1.headers == r2.headers && r1.body == r2.body
      }),
      _ => false,
    }
  }
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
  let headers = mv8.create_object();
  for (key, value) in resp.headers.iter() {
    let key = mv8.create_string(key.as_str());
    let value = mv8.create_string(value.to_str().unwrap());
    headers.set(key, value)?;
  }
  let body = mv8.create_string(String::from_utf8_lossy(resp.body.as_ref()).as_ref());
  obj.set("status", MValue::Number(status))?;
  obj.set("headers", MValue::Object(headers))?;
  obj.set("body", MValue::String(body))?;
  Ok(MValue::Object(obj))
}

fn from_req(req: reqwest::Request, mv8: MiniV8) -> mini_v8::Result<MValue> {
  let obj = mv8.create_object();
  let url = mv8.create_string(req.url().to_string().as_str());
  let method = mv8.create_string(req.method().clone().as_str());
  let headers = mv8.create_object();
  for (key, value) in req.headers().iter() {
    let key = mv8.create_string(key.as_str());
    let value = mv8.create_string(value.to_str().unwrap());
    headers.set(key, value)?;
  }
  if let Some(body) = req.body() {
    let body = mv8.create_string(String::from_utf8_lossy(body.as_bytes().unwrap()).as_ref());
    obj.set("body", MValue::String(body))?;
  }
  obj.set("url", MValue::String(url))?;
  obj.set("method", MValue::String(method))?;
  obj.set("headers", MValue::Object(headers))?;
  Ok(MValue::Object(obj))
}

fn to_req(value: MValue) -> mini_v8::Result<reqwest::Request> {
  let obj = value.as_object().expect("value is not an object");
  let url: String = obj.get::<&str, String>("url")?;
  let method: String = obj.get::<&str, String>("method")?;
  let body = obj.get::<&str, String>("body");
  let mut req = reqwest::Request::new(
    reqwest::Method::from_bytes(method.as_bytes()).unwrap(),
    url.parse().unwrap(),
  );
  let headers = obj.get::<&str, BTreeMap<String, String>>("headers");
  if let Ok(headers) = headers {
    let map = create_header_map(headers);
    req.headers_mut().extend(map);
  }
  let body = body.unwrap_or("".to_string());
  let _ = req.body_mut().insert(Body::from(body));
  Ok(req)
}

impl FromValue for Command {
  fn from_value(value: MValue, _mv8: &MiniV8) -> mini_v8::Result<Self> {
    match value {
      Value::Array(arr) => {
        let mut vec = Vec::new();
        for val in arr.elements() {
          vec.push(to_req(val?)?);
        }
        Ok(Command::Request(vec))
      }
      Value::Object(obj) => {
        let status = obj.get::<&str, f64>("status")?;
        let headers = obj.get::<&str, BTreeMap<String, String>>("headers")?;
        let map = create_header_map(headers);
        let body = obj.get::<&str, String>("body")?;
        let body = Bytes::from(body);
        let resp = Response { status: reqwest::StatusCode::from_u16(status as u16).unwrap(), headers: map, body };
        Ok(Command::Response(resp))
      }
      _ => unimplemented!("Command::from_value"),
    }
  }
}
fn create_header_map(headers: BTreeMap<String, String>) -> reqwest::header::HeaderMap {
  let mut header_map = reqwest::header::HeaderMap::new();
  for (key, value) in headers.iter() {
    let key = HeaderName::from_bytes(key.as_bytes()).unwrap();
    let value = HeaderValue::from_str(value.as_str()).unwrap();
    header_map.insert(key, value);
  }
  header_map
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

// mod test {
//
//   use mini_v8::{FromValue, MiniV8, ToValues, Value};
//
//   use crate::{Command, Event};
//
//   #[test]
//   fn test_codec() {
//     let mv8 = MiniV8::new();
//     let request = reqwest::Request::new(
//       reqwest::Method::GET,
//       reqwest::Url::parse("http://localhost:8000").unwrap(),
//     );
//     let event = Event::Request(request);
//     let value = event.to_values(&mv8).unwrap().into_vec();
//     let mut array = mv8.create_array();
//     for val in value {
//       array.push(val).unwrap();
//     }
//     let value = Value::Array(array);
//     let cmd = Command::from_value(value, &mv8).unwrap();
//
//     println!("{:?}", cmd);
//   }
// }
