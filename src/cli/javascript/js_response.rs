use std::collections::BTreeMap;

use hyper::body::Bytes;
use serde::{Deserialize, Serialize};

use super::create_header_map;
use crate::http::Response;
use crate::is_default;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JsResponse {
    status: u16,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    headers: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "is_default")]
    body: Option<Bytes>,
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

#[cfg(test)]
mod test {
    use anyhow::Result;
    use hyper::body::Bytes;
    use reqwest::header::HeaderMap;

    use super::JsResponse;

    fn create_test_response() -> Result<JsResponse> {
        let mut headers = HeaderMap::new();
        headers.insert("content-type", "application/json".parse().unwrap());
        let response = crate::http::Response {
            status: reqwest::StatusCode::OK,
            headers,
            body: Bytes::from("Hello, World!"),
        };
        let js_response: Result<JsResponse> = response.try_into();
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
