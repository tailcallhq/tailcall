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
