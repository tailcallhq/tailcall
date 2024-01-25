use std::collections::BTreeMap;

use hyper::body::Bytes;
use hyper::header::{HeaderName, HeaderValue};
use reqwest::Request;

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
impl TryFrom<JsResponse> for Response<Bytes> {
  type Error = anyhow::Error;

  fn try_from(res: JsResponse) -> Result<Self, Self::Error> {
    let status = reqwest::StatusCode::from_u16(res.status as u16)?;
    let headers = create_header_map(res.headers)?;
    let body = serde_json::to_string(&res.body)?;
    Ok(Response { status, headers, body: Bytes::from(body) })
  }
}

impl TryFrom<&Response<Bytes>> for JsResponse {
  type Error = anyhow::Error;

  fn try_from(res: &Response<Bytes>) -> Result<Self, Self::Error> {
    let status = res.status.as_u16() as f64;
    let mut headers = BTreeMap::new();
    for (key, value) in res.headers.iter() {
      let key = key.to_string();
      let value = value.to_str()?.to_string();
      headers.insert(key, value);
    }

    let body = serde_json::from_slice(res.body.as_ref())?;
    Ok(JsResponse { status, headers, body })
  }
}

// Request implementations
impl TryFrom<JsRequest> for reqwest::Request {
  type Error = anyhow::Error;

  fn try_from(req: JsRequest) -> Result<Self, Self::Error> {
    let mut request = reqwest::Request::new(reqwest::Method::from_bytes(req.method.as_bytes())?, req.url.parse()?);
    let headers = create_header_map(req.headers)?;
    request.headers_mut().extend(headers);
    let body = serde_json::to_string(&req.body)?;
    let _ = request.body_mut().insert(reqwest::Body::from(body));
    Ok(request)
  }
}

impl TryFrom<&reqwest::Request> for JsRequest {
  type Error = anyhow::Error;

  fn try_from(req: &Request) -> Result<Self, Self::Error> {
    let url = req.url().to_string();
    let method = req.method().as_str().to_string();
    let mut headers = BTreeMap::new();
    for (key, value) in req.headers().iter() {
      let key = key.to_string();
      let value = value.to_str()?.to_string();
      headers.insert(key, value);
    }
    if let Some(body) = req.body() {
      let body = serde_json::from_slice(body.as_bytes().unwrap_or_default())?;
      Ok(JsRequest { url, method, headers, body })
    } else {
      Ok(JsRequest { url, method, headers, body: serde_json::Value::Null })
    }
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

fn create_header_map(headers: BTreeMap<String, String>) -> anyhow::Result<reqwest::header::HeaderMap> {
  let mut header_map = reqwest::header::HeaderMap::new();
  for (key, value) in headers.iter() {
    let key = HeaderName::from_bytes(key.as_bytes())?;
    let value = HeaderValue::from_str(value.as_str())?;
    header_map.insert(key, value);
  }
  Ok(header_map)
}
