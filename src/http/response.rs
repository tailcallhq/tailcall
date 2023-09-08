use derive_setters::Setters;

use http_cache_semantics::ResponseLike;

#[derive(Clone, Debug, Default, Setters)]
pub struct Response {
    pub status: reqwest::StatusCode,
    pub headers: reqwest::header::HeaderMap,
    pub body: async_graphql::Value,
    pub ttl: Option<u64>,
}
impl From<&reqwest::Response> for Response {
    fn from(resp: &reqwest::Response) -> Self {
        let status = resp.status();
        let headers = resp.headers().clone();
        let body = async_graphql::Value::Null;
        let ttl = None;
        Response { status, headers, body, ttl }
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
