use derive_setters::Setters;

use http_cache_semantics::RequestLike;
use hyper::Uri;
use reqwest::header::HeaderMap;

#[derive(Clone, Debug, Default, Setters)]
pub struct Request {
    pub url: String,
    pub method: reqwest::Method,
    pub headers: reqwest::header::HeaderMap,
    pub body: String,
}

impl From<&reqwest::Request> for Request {
    fn from(req: &reqwest::Request) -> Self {
        let url = req.url().to_string();
        let method = req.method().clone();
        let headers = req.headers().clone();
        let body = String::new();
        Request { url, method, headers, body }
    }
}

impl RequestLike for Request {
    fn uri(&self) -> Uri {
        self.url.parse().unwrap()
    }

    fn is_same_uri(&self, other: &Uri) -> bool {
        self.url == (*other).to_string()
    }

    fn method(&self) -> &reqwest::Method {
        &self.method
    }

    fn headers(&self) -> &HeaderMap {
        &self.headers
    }
}
