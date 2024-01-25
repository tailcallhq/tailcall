use crate::http::Response;
use hyper::body::Bytes;
use hyper::header::{HeaderName, HeaderValue};
use std::collections::BTreeMap;

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
