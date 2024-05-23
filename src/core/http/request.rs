use hyper::{Method, Uri, Version};
use anyhow::Result;
use derive_setters::Setters;
use hyper::http::request::{Builder, Parts};
use http_body_util::BodyExt;
use hyper::header::HeaderMap;

#[derive(Setters, Default)]
pub struct Request {
    pub method: Method,
    pub uri: Uri,
    pub version: Version,
    pub headers: HeaderMap,
    pub body: bytes::Bytes,
}

impl Request {
    pub fn builder() -> Self {
        Self::default()
    }
    pub async fn from_hyper(req: hyper::Request<hyper::body::Incoming>) -> Result<Self> {
        let (parts, body) = req.into_parts();
        let body = body.collect().await?.to_bytes();
        Ok(
            Request {
                method: parts.method,
                uri: parts.uri,
                version: parts.version,
                headers: parts.headers,
                body,
            }
        )
    }
    pub fn parts(&self) -> Parts {
        let parts = Builder::new()
            .method(self.method.clone())
            .uri(self.uri.clone())
            .version(self.version)
            .body(())
            .unwrap()
            .into_parts();
        parts.0
    }
}