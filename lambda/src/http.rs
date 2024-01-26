use anyhow::Result;
use hyper::body::Bytes;
use reqwest::Client;
use tailcall::http::Response;
use tailcall::HttpIO;
use tokio::task::spawn_local;

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
        let res = spawn_local(async move {
            let response = client.execute(request).await?.error_for_status()?;
            Response::from_reqwest(response).await
        })
        .await??;
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
    let body = hyper::Body::from(req.body().as_ref().to_owned());
    hyper::Request::new(body)
}
