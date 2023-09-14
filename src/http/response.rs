use derive_setters::Setters;

use http_cache_semantics::ResponseLike;

use super::stats::Stats;

#[derive(Clone, Debug, Default, Setters)]
pub struct Response {
    pub status: reqwest::StatusCode,
    pub headers: reqwest::header::HeaderMap,
    pub body: async_graphql::Value,
    pub stats: Stats,
}

impl Response {
    pub fn min_ttl(mut self, value: u64) -> Self {
        self.stats.min_ttl = Some(value);
        self
    }
}

// FIXME: embed body in Response
impl From<&reqwest::Response> for Response {
    fn from(resp: &reqwest::Response) -> Self {
        let status = resp.status();
        let headers = resp.headers().clone();
        let body = async_graphql::Value::Null;
        Response { status, headers, body, stats: Stats::default() }
    }
}

impl ResponseLike for Response {
    fn status(&self) -> reqwest::StatusCode {
        self.status
    }

    fn headers(&self) -> &reqwest::header::HeaderMap {
        &self.headers
    }
}
