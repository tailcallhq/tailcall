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
        let req_str = format!("{} {}", request.method(), request.url());
        let response = self.client.execute(request).await?.error_for_status()?;
        let res = Response::from_reqwest(response).await?;
        tracing::info!("{} {}", req_str, res.status.as_u16());
        Ok(res)
    }
}

pub fn to_request(
    req: lambda_http::Request,
) -> Result<hyper::Request<hyper::Body>, hyper::http::Error> {
    // TODO: Update hyper to 1.0 to make conversions easier
    let method: hyper::Method = match req.method().to_owned() {
        lambda_http::http::Method::CONNECT => hyper::Method::CONNECT,
        lambda_http::http::Method::DELETE => hyper::Method::DELETE,
        lambda_http::http::Method::GET => hyper::Method::GET,
        lambda_http::http::Method::HEAD => hyper::Method::HEAD,
        lambda_http::http::Method::OPTIONS => hyper::Method::OPTIONS,
        lambda_http::http::Method::PATCH => hyper::Method::PATCH,
        lambda_http::http::Method::POST => hyper::Method::POST,
        lambda_http::http::Method::PUT => hyper::Method::PUT,
        lambda_http::http::Method::TRACE => hyper::Method::TRACE,
        _ => unreachable!(),
    };

    hyper::Request::builder()
        .method(method)
        .uri::<String>(req.uri().to_string())
        .body(hyper::Body::from(req.body().to_vec()))
}

pub async fn to_response(
    res: hyper::Response<hyper::Body>,
) -> Result<lambda_http::Response<lambda_http::Body>, lambda_http::http::Error> {
    // TODO: Update hyper to 1.0 to make conversions easier
    let mut build = lambda_http::Response::builder().status(res.status().as_u16());

    for (k, v) in res.headers() {
        build = build.header(k.to_string(), v.as_bytes());
    }

    build.body(lambda_http::Body::Binary(Vec::from(
        hyper::body::to_bytes(res.into_body()).await.unwrap(),
    )))
}

pub fn init_http() -> Arc<LambdaHttp> {
    Arc::new(LambdaHttp::init())
}
