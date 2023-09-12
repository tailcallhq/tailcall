use http_cache_semantics::RequestLike;
use hyper::Uri;
use reqwest::header::HeaderMap;

// TODO: why do we need clone?
#[derive(Debug)]
pub struct Request<'a>(&'a reqwest::Request);

impl <'a> Request<'a> {
    pub fn to_reqwest(self) -> &'a reqwest::Request {
        self.0
    }
    pub fn url(&self) -> &reqwest::Url {
        self.0.url()
    }
    pub fn method(&self) -> &reqwest::Method {
        self.0.method()
    }
    pub fn headers(&self) -> &HeaderMap {
        self.0.headers()
    }
}

impl<'a> From<&'a reqwest::Request> for Request<'a> {
    fn from(req: &'a reqwest::Request) -> Self {
        Request(req)
    }
}

impl <'a>RequestLike for Request<'a> {
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
