use headers::HeaderMap;
use hyper::{Body, Response};

#[derive(Debug, Clone)]
pub struct TailcallResponse {
    headers: HeaderMap,
    body: hyper::body::Bytes,
}

impl TailcallResponse {
    pub fn into_response(mut self) -> anyhow::Result<Response<Body>> {
        let mut resp = Response::new(Body::from(self.body));
        std::mem::swap(resp.headers_mut(), &mut self.headers);
        Ok(resp)
    }

    pub fn new(headers: HeaderMap, body: hyper::body::Bytes) -> Self {
        Self { headers, body }
    }
}
