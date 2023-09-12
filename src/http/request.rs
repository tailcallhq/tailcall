use http_cache_semantics::RequestLike;
use hyper::Uri;
use reqwest::header::HeaderMap;

#[derive(Debug)]
pub struct Request<'a> {
    inner: &'a reqwest::Request,
}

impl<'a> Request<'a> {
    pub fn url(&self) -> &reqwest::Url {
        self.inner.url()
    }
    pub fn method(&self) -> &reqwest::Method {
        self.inner.method()
    }
    pub fn headers(&self) -> &HeaderMap {
        self.inner.headers()
    }
}

impl<'a> From<&'a reqwest::Request> for Request<'a> {
    fn from(inner: &'a reqwest::Request) -> Self {
        Request { inner }
    }
}

impl<'a> RequestLike for Request<'a> {
    fn uri(&self) -> Uri {
        self.url().as_str().parse().unwrap()
    }

    fn is_same_uri(&self, other: &Uri) -> bool {
        self.url().to_string() == (*other).to_string()
    }

    fn method(&self) -> &reqwest::Method {
        self.method()
    }

    fn headers(&self) -> &HeaderMap {
        self.headers()
    }
}
