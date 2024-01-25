pub use js_http::*;
pub use v8_value::*;

mod v8_value {
  use mini_v8::{MiniV8, Value};
  use serde::de::DeserializeOwned;
  use serde::Serialize;
  use serde_json::Number;

  pub trait SerdeV8: Sized {
    fn to_v8(self, mv8: &MiniV8) -> anyhow::Result<Value>;
    fn from_v8(value: &Value) -> anyhow::Result<Self>;
  }

  fn v8_serde(value: mini_v8::Value) -> anyhow::Result<serde_json::Value> {
    let serde_value: serde_json::Value = match value {
      Value::Undefined => serde_json::Value::Null,
      Value::Null => serde_json::Value::Null,
      Value::Boolean(v) => serde_json::Value::Bool(v),
      Value::Number(n) => {
        serde_json::Value::Number(Number::from_f64(n).ok_or(anyhow::anyhow!("error converting number"))?)
      }
      Value::String(s) => serde_json::Value::String(s.to_string()),
      Value::Array(v) => {
        let mut arr = Vec::new();
        for v in v.elements::<Value>() {
          arr.push(v8_serde(v.map_err(|e| anyhow::anyhow!(e.to_string()))?)?);
        }
        serde_json::Value::Array(arr)
      }
      Value::Function(_) => serde_json::Value::Null,
      Value::Object(v) => {
        let mut obj = serde_json::Map::new();
        let props = v.properties(false).map_err(|e| anyhow::anyhow!(e.to_string()))?;
        for kv in props {
          let (k, v) = kv.map_err(|e| anyhow::anyhow!(e.to_string()))?;
          obj.insert(k, v8_serde(v)?);
        }
        serde_json::Value::Object(obj)
      }
      Value::Date(d) => serde_json::Value::Number(Number::from_f64(d).ok_or(anyhow::anyhow!("error converting date"))?),
    };

    Ok(serde_value)
  }

  fn serde_v8(value: serde_json::Value, v8: &mini_v8::MiniV8) -> anyhow::Result<mini_v8::Value> {
    let value: mini_v8::Value = match value {
      serde_json::Value::Null => Value::Null,
      serde_json::Value::Bool(b) => Value::Boolean(b),
      serde_json::Value::Number(n) => Value::Number(n.as_f64().unwrap_or_default()),
      serde_json::Value::String(s) => Value::String(v8.create_string(s.as_str())),
      serde_json::Value::Array(a) => {
        let arr = v8.create_array();
        for v in a {
          arr.push(serde_v8(v, v8)?).map_err(|e| anyhow::anyhow!(e.to_string()))?;
        }
        Value::Array(arr)
      }
      serde_json::Value::Object(obj) => {
        let out = v8.create_object();
        for (k, v) in obj {
          out
            .set(k, serde_v8(v, v8)?)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        }
        Value::Object(out)
      }
    };
    Ok(value)
  }

  impl<A: Serialize + DeserializeOwned> SerdeV8 for A {
    fn to_v8(self, mv8: &MiniV8) -> anyhow::Result<Value> {
      let json = serde_json::to_value(&self)?;
      log::debug!("json: {}", json);
      serde_v8(json, mv8)
    }

    fn from_v8(value: &Value) -> anyhow::Result<A> {
      let serde_value = v8_serde(value.clone())?;
      let value: A = serde_json::from_value(serde_value)?;
      Ok(value)
    }
  }
}

mod js_http {
  use std::collections::BTreeMap;

  use hyper::body::Bytes;
  use hyper::header::{HeaderName, HeaderValue};

  use crate::http::Response;

  #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
  #[serde(rename_all = "camelCase")]
  pub enum Event {
    Request(JsRequest),
    Response(Vec<JsResponse>),
  }

  #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
  #[serde(rename_all = "camelCase")]
  pub enum Command {
    Request(Vec<JsRequest>),
    Response(JsResponse),
    Continue(JsRequest),
  }

  #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
  #[serde(rename_all = "camelCase")]
  pub struct JsRequest {
    url: String,
    method: String,
    headers: BTreeMap<String, String>,
    body: serde_json::Value,
  }

  #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
  #[serde(rename_all = "camelCase")]
  pub struct JsResponse {
    status: f64,
    headers: BTreeMap<String, String>,
    body: serde_json::Value,
  }

  // Response implementations
  impl From<JsResponse> for Response<Bytes> {
    fn from(res: JsResponse) -> Self {
      let status = reqwest::StatusCode::from_u16(res.status as u16).unwrap();
      let headers = create_header_map(res.headers);
      let body = serde_json::to_string(&res.body).unwrap();
      Response { status, headers, body: Bytes::from(body) }
    }
  }

  impl From<&Response<Bytes>> for JsResponse {
    fn from(res: &Response<Bytes>) -> Self {
      let status = res.status.as_u16() as f64;
      let headers = res
        .headers
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap().to_string()))
        .collect();
      let body = serde_json::from_slice(res.body.as_ref()).unwrap();
      JsResponse { status, headers, body }
    }
  }

  // Request implementations
  impl From<JsRequest> for reqwest::Request {
    fn from(req: JsRequest) -> Self {
      let mut request = reqwest::Request::new(
        reqwest::Method::from_bytes(req.method.as_bytes()).unwrap(),
        req.url.parse().unwrap(),
      );
      let headers = create_header_map(req.headers);
      request.headers_mut().extend(headers);
      let body = serde_json::to_string(&req.body).unwrap();
      let _ = request.body_mut().insert(reqwest::Body::from(body));
      request
    }
  }

  impl From<&reqwest::Request> for JsRequest {
    fn from(req: &reqwest::Request) -> Self {
      let url = req.url().to_string();
      let method = req.method().as_str().to_string();
      let headers = req
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap().to_string()))
        .collect();
      let body = req
        .body()
        .map(|b| serde_json::from_slice::<serde_json::Value>(b.as_bytes().unwrap_or_default()).unwrap())
        .unwrap_or_default();
      JsRequest { url, method, headers, body }
    }
  }

  impl Event {
    pub fn response(&self) -> Vec<JsResponse> {
      match self {
        Event::Response(res) => res.clone(),
        _ => Vec::new(),
      }
    }
    pub fn request(&self) -> Option<JsRequest> {
      match self {
        Event::Request(req) => Some(req.clone()),
        _ => None,
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
}
