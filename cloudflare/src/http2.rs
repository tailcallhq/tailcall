use hyper::body::Bytes;
use tailcall::http::Response;
use tailcall::HttpIO;

#[derive(Clone)]
pub struct CloudflareHttp2 {}

impl CloudflareHttp2 {
    pub fn init() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl HttpIO for CloudflareHttp2 {
    async fn execute(&self, _request: reqwest::Request) -> anyhow::Result<Response<Bytes>> {
        Ok(Response::default())
    }
}
