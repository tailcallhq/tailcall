use http_cache_semantics::RequestLike;
use hyper::Uri;
use reqwest::header::HeaderMap;

/// A specialized request for GET requests.
/// This is created to reduce allocations.
#[derive(Clone, Debug)]
pub struct GetRequest {
    url: reqwest::Url,
    headers: reqwest::header::HeaderMap,
}

impl From<&reqwest::Request> for GetRequest {
    fn from(inner: &reqwest::Request) -> Self {
        GetRequest { url: inner.url().clone(), headers: inner.headers().clone() }
    }
}

impl RequestLike for GetRequest {
    fn uri(&self) -> Uri {
        self.url.as_str().parse().expect("Uri and Url are incompatible!?")
    }

    fn is_same_uri(&self, other: &Uri) -> bool {
        self.uri() == *other
    }

    fn method(&self) -> &reqwest::Method {
        &reqwest::Method::GET
    }

    fn headers(&self) -> &HeaderMap {
        &self.headers
    }
}
