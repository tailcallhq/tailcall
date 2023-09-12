use derive_setters::Setters;

use http_cache_semantics::ResponseLike;

#[derive(Debug, Setters)]
pub struct Response<'a> {
    pub inner: &'a reqwest::Response,
    pub stats: Stats,
}

#[derive(Debug, Default)]
struct Stats {
    ttl: Option<u64>,
}

impl<'a> From<&'a reqwest::Response> for Response<'a> {
    fn from(inner: &reqwest::Response) -> Self {
        Response { inner, stats: Stats::default() }
    }
}

impl<'a> ResponseLike for Response<'a> {
    fn status(&self) -> reqwest::StatusCode {
        self.inner.status()
    }

    fn headers(&self) -> &reqwest::header::HeaderMap {
        self.inner.headers()
    }
}
