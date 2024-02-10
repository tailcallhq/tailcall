use std::collections::BTreeMap;

use hyper::body::Bytes;
use serde::{Deserialize, Serialize};

use super::create_header_map;
use crate::is_default;
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

impl TryFrom<&reqwest::Request> for JsRequest {
    type Error = anyhow::Error;

    fn try_from(req: &reqwest::Request) -> Result<Self, Self::Error> {
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

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_js_request_to_reqwest_request() {
        let body = "Hello, World!";
        let mut headers = BTreeMap::new();
        headers.insert("x-unusual-header".to_string(), "🚀".to_string());

        let js_request = JsRequest {
            url: "http://example.com/".to_string(),
            method: "GET".to_string(),
            headers,
            body: Some(Bytes::from(body)),
        };
        let reqwest_request: reqwest::Request = js_request.try_into().unwrap();
        assert_eq!(reqwest_request.method(), reqwest::Method::GET);
        assert_eq!(reqwest_request.url().as_str(), "http://example.com/");
        assert_eq!(
            reqwest_request.headers().get("x-unusual-header").unwrap(),
            "🚀"
        );
        let body_out = reqwest_request
            .body()
            .as_ref()
            .and_then(|body| body.as_bytes())
            .map(|a| String::from_utf8_lossy(a).to_string());
        assert_eq!(body_out, Some(body.to_string()));
    }

    #[test]
    fn test_reqwest_request_to_js_request() {
        let mut reqwest_request =
            reqwest::Request::new(reqwest::Method::GET, "http://example.com/".parse().unwrap());
        let _ = reqwest_request
            .body_mut()
            .insert(reqwest::Body::from("Hello, World!"));
        let js_request: JsRequest = (&reqwest_request).try_into().unwrap();
        assert_eq!(js_request.method, "GET");
        assert_eq!(js_request.url, "http://example.com/");
        let body_out = js_request.body.unwrap();
        assert_eq!(body_out, Bytes::from("Hello, World!"));
    }
}
