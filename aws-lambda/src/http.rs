use std::sync::Arc;

use anyhow::Result;
use hyper::body::Bytes;
use reqwest::Client;
use tailcall::http::Response;
use tailcall::HttpIO;

#[derive(Clone)]
pub struct LambdaHttp {
    client: Client,
}

impl Default for LambdaHttp {
    fn default() -> Self {
        Self { client: Client::new() }
    }
}

impl LambdaHttp {
    pub fn init() -> Self {
        Default::default()
    }
}

#[async_trait::async_trait]
impl HttpIO for LambdaHttp {
    async fn execute(&self, request: reqwest::Request) -> Result<Response<Bytes>> {
        let client = self.client.clone();
        let req_str = format!("{} {}", request.method(), request.url());
        let response = client.execute(request).await?;
        let res = Response::from_reqwest(response).await?;
        tracing::info!("{} {}", req_str, res.status.as_u16());
        Ok(res)
    }
}

pub fn to_request(
    req: lambda_http::Request,
) -> Result<hyper::Request<hyper::Body>, lambda_http::http::Error> {
    hyper::Request::builder()
        .method(req.method())
        .uri(req.uri())
        .body(hyper::Body::from(req.body().to_vec()))
}

pub fn init_http() -> Arc<LambdaHttp> {
    Arc::new(LambdaHttp::init())
}
