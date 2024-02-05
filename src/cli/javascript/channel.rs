use std::collections::BTreeMap;

use hyper::body::Bytes;
use hyper::header::{HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};

use crate::http::Response;
use crate::is_default;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub message: MessageContent,
    pub id: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum MessageContent {
    Request(JsRequest),
    Response(JsResponse),
    Empty,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JsRequest {
    url: String,
    method: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    headers: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "is_default")]
    body: Option<Bytes>,
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JsResponse {
    status: u16,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    headers: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "is_default")]
    body: Option<Bytes>,
}

impl TryFrom<JsRequest> for reqwest::Request {
    type Error = anyhow::Error;

    fn try_from(req: JsRequest) -> Result<Self, Self::Error> {
        let mut request = reqwest::Request::new(
            reqwest::Method::from_bytes(req.method.as_bytes())?,
            req.url.parse()?,
        );
        let headers = create_header_map(req.headers)?;
        request.headers_mut().extend(headers);
        if let Some(bytes) = req.body {
            let _ = request.body_mut().insert(reqwest::Body::from(bytes));
        }

        Ok(request)
    }
}

impl TryFrom<reqwest::Request> for JsRequest {
    type Error = anyhow::Error;

    fn try_from(req: reqwest::Request) -> Result<Self, Self::Error> {
        let url = req.url().to_string();
        let method = req.method().as_str().to_string();
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
        let body = req.body().map(|body| {
            let bytes = body.as_bytes().unwrap_or_default();
            Bytes::from_iter(bytes.to_vec())
        });

        Ok(JsRequest { url, method, headers, body })
    }
}

impl TryFrom<JsResponse> for Response<Bytes> {
    type Error = anyhow::Error;

    fn try_from(res: JsResponse) -> Result<Self, Self::Error> {
        let status = reqwest::StatusCode::from_u16(res.status)?;
        let headers = create_header_map(res.headers)?;
        let body = res.body.unwrap_or_default();
        Ok(Response { status, headers, body })
    }
}

impl TryFrom<Response<Bytes>> for JsResponse {
    type Error = anyhow::Error;

    fn try_from(res: Response<Bytes>) -> Result<Self, Self::Error> {
        let status = res.status.as_u16();
        let mut headers = BTreeMap::new();
        for (key, value) in res.headers.iter() {
            let key = key.to_string();
            let value = value.to_str()?.to_string();
            headers.insert(key, value);
        }

        let body = Some(res.body);
        Ok(JsResponse { status, headers, body })
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

#[cfg(test)]
mod test {
    use anyhow::Result;
    use hyper::body::Bytes;
    use reqwest::header::HeaderMap;

    use crate::cli::javascript::channel::JsResponse;

    fn create_test_response() -> Result<JsResponse> {
        let mut headers = HeaderMap::new();
        headers.insert("content-type", "application/json".parse().unwrap());
        let response = crate::http::Response {
            status: reqwest::StatusCode::OK,
            headers,
            body: Bytes::from("Hello, World!"),
        };
        let js_response: Result<crate::cli::javascript::channel::JsResponse> = response.try_into();
        js_response
    }
    #[test]
    fn test_to_js_response() {
        let js_response = create_test_response();
        println!("{:?}", js_response);
        assert!(js_response.is_ok());
        let js_response = js_response.unwrap();
        assert_eq!(js_response.status, 200);
        assert_eq!(
            js_response.headers.get("content-type").unwrap(),
            "application/json"
        );
        assert_eq!(js_response.body, Some("Hello, World!".as_bytes().into()));
    }

    #[test]
    fn test_from_js_response() {
        let js_response = create_test_response().unwrap();
        let response: Result<crate::http::Response<Bytes>> = js_response.try_into();
        assert!(response.is_ok());
        let response = response.unwrap();
        assert_eq!(response.status, reqwest::StatusCode::OK);
        assert_eq!(
            response.headers.get("content-type").unwrap(),
            "application/json"
        );
        assert_eq!(response.body, Bytes::from("Hello, World!"));
    }
}
