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
        let method = request.method().clone();
        let url = request.url().clone();
        let response = client.execute(request).await?.error_for_status()?;
        let res = Response::from_reqwest(response).await?;
        tracing::info!("{} {} {}", method, url, res.status.as_u16());
        Ok(res)
    }
}

pub fn to_response(
    response: hyper::Response<hyper::Body>,
) -> anyhow::Result<lambda_http::Response<hyper::Body>> {
    Ok(response)
}

pub fn to_request(req: lambda_http::Request) -> hyper::Request<hyper::Body> {
    let builder = hyper::Request::builder()
        .method(req.method())
        .uri(req.uri())
        .body(hyper::Body::from(req.body().as_ref().to_vec()));

    builder.unwrap()
}

pub fn init_http() -> Arc<LambdaHttp> {
    Arc::new(LambdaHttp::init())
}
