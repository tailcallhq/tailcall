pub use js_http::*;
pub use v8_value::*;

mod v8_value {
  use mini_v8::{MiniV8, Value};

  pub trait SerdeV8 {
    fn to_v8(self, mv8: &MiniV8) -> anyhow::Result<Value>;
    fn from_v8(value: Value) -> anyhow::Result<serde_json::Value>;
  }

  impl SerdeV8 for serde_json::Value {
    fn to_v8(self, mv8: &MiniV8) -> anyhow::Result<Value> {
      let json = serde_json::to_string(&self)?;
      let value = mv8.create_string(json.as_str());
      Ok(Value::String(value))
    }

    fn from_v8(value: Value) -> anyhow::Result<Self> {
      match value.as_string() {
        Some(json) => Ok(serde_json::from_str(json.to_string().as_str())?),
        None => Err(anyhow::anyhow!("value is not a valid json string")),
      }
    }
  }

  // impl ToValue for Response<Bytes> {
  //   fn to_value(self, mv8: &MiniV8) -> mini_v8::Result<Value> {
  //     from_resp(self, mv8.clone())
  //   }
  // }

  // pub struct ValueWrapper(pub serde_json::Value);
  // impl From<ValueWrapper> for serde_json::Value {
  //   fn from(value: ValueWrapper) -> Self {
  //     value.0
  //   }
  // }

  // impl FromValue for ValueWrapper {
  //   fn from_value(value: Value, _mv8: &MiniV8) -> mini_v8::Result<Self> {
  //     let p = match value {
  //       Value::Undefined => serde_json::Value::Null,
  //       Value::Null => serde_json::Value::Null,
  //       Value::Boolean(v) => serde_json::Value::Bool(v),
  //       Value::Number(n) => serde_json::Value::Number(Number::from_f64(n).ok_or(Error::FromJsConversionError {
  //         from: "number",
  //         to: "graphql number as it is out of supported range",
  //       })?),

  //       Value::String(s) => serde_json::Value::String(s.to_string()),
  //       Value::Array(v) => {
  //         let list: mini_v8::Result<Vec<serde_json::Value>> =
  //           v.elements::<ValueWrapper>().map(|e| e.map(|v| v.into())).collect();

  //         serde_json::Value::Array(list?)
  //       }
  //       Value::Function(_) => serde_json::Value::Null,
  //       Value::Object(v) => {
  //         let props: mini_v8::Result<Vec<(String, serde_json::Value)>> = v
  //           .properties::<String, ValueWrapper>(false)?
  //           .map(|e| e.map(|(k, v)| (k, v.into())))
  //           .collect();

  //         serde_json::Value::Object(serde_json::Map::from_iter(props?))
  //       }

  //       Value::Date(d) => serde_json::Value::Number(Number::from_f64(d).ok_or(Error::FromJsConversionError {
  //         from: "Date",
  //         to: "graphql number as it is out of supported range",
  //       })?),
  //     };
  //     Ok(ValueWrapper(p))
  //   }
  // }

  // impl ToValue for ValueWrapper {
  //   fn to_value(self, mv8: &MiniV8) -> mini_v8::Result<Value> {
  //     let p = match self.0 {
  //       serde_json::Value::Null => Value::Null,
  //       serde_json::Value::Bool(b) => Value::Boolean(b),
  //       serde_json::Value::Number(n) => Value::Number(n.as_f64().unwrap_or_default()),
  //       serde_json::Value::String(s) => Value::String(mv8.create_string(s.as_str())),
  //       serde_json::Value::Array(a) => {
  //         let arr = mv8.create_array();
  //         for v in a {
  //           arr.push(ValueWrapper(v).to_value(mv8)?)?;
  //         }
  //         Value::Array(arr)
  //       }
  //       serde_json::Value::Object(obj) => {
  //         let out = mv8.create_object();
  //         for (k, v) in obj {
  //           out.set(k, ValueWrapper(v).to_value(mv8)?)?;
  //         }
  //         Value::Object(out)
  //       }
  //     };
  //     Ok(p)
  //   }
  // }

  // fn from_req(req: JsRequest, mv8: MiniV8) -> mini_v8::Result<Value> {
  //   let serde_value = serde_json::to_value(req).unwrap();
  //   let value = ValueWrapper(serde_value);
  //   let obj = value.to_value(&mv8)?;
  //   let out = mv8.create_object();
  //   out.set("request", obj)?;
  //   Ok(Value::Object(out))
  // }

  // impl FromValue for Command {
  //   fn from_value(value: Value, _mv8: &MiniV8) -> mini_v8::Result<Self> {
  //     let serde_value: serde_json::Value = ValueWrapper::from_value(value, _mv8)?.into();
  //     let command: Command = serde_json::from_value(serde_value)
  //       .map_err(|_e| Error::FromJsConversionError { from: "serde_json::Value", to: "Command" })?;
  //     Ok(command)
  //   }
  // }

  // impl ToValues for Event {
  //   fn to_values(self, mv8: &MiniV8) -> mini_v8::Result<Values> {
  //     match self {
  //       Event::Request(req) => {
  //         let req = from_req(req, mv8.clone())?;
  //         let val = Values::from_iter(vec![req]);
  //         Ok(val)
  //       }
  //       Event::Response(responses) => {
  //         let arr = mv8.create_array();

  //         for res in responses {
  //           arr.push(from_resp(res.into(), mv8.clone())?)?;
  //         }
  //         let out = mv8.create_object();
  //         out.set("responses", Value::Array(arr))?;
  //         Ok(Values::from_iter(vec![Value::Object(out)]))
  //       }
  //     }
  //   }
  // }

  // fn from_resp(resp: Response<Bytes>, mv8: MiniV8) -> mini_v8::Result<Value> {
  //   let resp = JsResponse::from(&resp);
  //   let serde_value = serde_json::to_value(resp).unwrap();
  //   let value = ValueWrapper(serde_value);
  //   let obj = value.to_value(&mv8)?;
  //   Ok(obj)
  // }
}

mod js_http {
  use std::collections::BTreeMap;

  use hyper::body::Bytes;
  use hyper::header::{HeaderName, HeaderValue};

  use crate::http::Response;

  #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
  pub enum Event {
    Request(JsRequest),
    Response(Vec<JsResponse>),
  }

  #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
  pub enum Command {
    #[serde(rename = "request")]
    Request(Vec<JsRequest>),
    #[serde(rename = "response")]
    Response(JsResponse),
    #[serde(rename = "continue")]
    Continue(JsRequest),
  }

  #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
  pub struct JsRequest {
    url: String,
    method: String,
    headers: BTreeMap<String, String>,
    body: serde_json::Value,
  }

  #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
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
