use std::collections::BTreeMap;

use hyper::body::Bytes;
use hyper::header::{HeaderName, HeaderValue};
use reqwest::Request;

use crate::http::{Method, Response};
use crate::is_default;

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
    #[serde(default, skip_serializing_if = "is_default")]
    method: Method,
    #[serde(default, skip_serializing_if = "is_default")]
    headers: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "is_default")]
    body: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsResponse {
    #[serde(
        default = "default_http_status",
        skip_serializing_if = "is_default_status"
    )]
    pub status: f64, // TODO: make this u16 once we derive the codecs for v8::Value
    #[serde(default, skip_serializing_if = "is_default")]
    pub headers: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub body: Option<serde_json::Value>,
}

fn default_http_status() -> f64 {
    200.0
}

fn is_default_status(status: &f64) -> bool {
    *status == default_http_status()
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
        let mut request = reqwest::Request::new(req.method.to_hyper(), req.url.parse()?);
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
        let method = Method::from(req.method().as_str());
        let headers = req
            .headers()
            .iter()
            .map(|(key, value)| {
                (
                    key.to_string(),
                    value.to_str().unwrap_or_default().to_string(),
                )
            })
            .collect::<BTreeMap<String, String>>();
        let body = req
            .body()
            .and_then(|body| body.as_bytes())
            .and_then(|body| serde_json::from_slice(body).ok());
        Ok(JsRequest { url, method, headers, body })
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

fn create_header_map(
    headers: BTreeMap<String, String>,
) -> anyhow::Result<reqwest::header::HeaderMap> {
    let mut header_map = reqwest::header::HeaderMap::new();
    for (key, value) in headers.iter() {
        let key = HeaderName::from_bytes(key.as_bytes())?;
        let value = HeaderValue::from_str(value.as_str())?;
        header_map.insert(key, value);
    }
    Ok(header_map)
}
