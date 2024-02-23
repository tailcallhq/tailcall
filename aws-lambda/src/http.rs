use std::sync::Arc;

use anyhow::{Context, Result};
use http_body_util::{BodyExt, Full};
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
) -> Result<hyper::Request<Full<Bytes>>, hyper::http::Error> {
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
        .body(Full::new(Bytes::from(req.body().to_vec())))
}

pub async fn to_response(
    res: hyper::Response<Full<Bytes>>,
) -> Result<lambda_http::Response<lambda_http::Body>> {
    // TODO: Update hyper to 1.0 to make conversions easier
    let mut build = lambda_http::Response::builder().status(res.status().as_u16());

    for (k, v) in res.headers() {
        build = build.header(k.to_string(), v.as_bytes());
    }

    let bytes = res
        .into_body()
        .frame()
        .await
        .context("unable to extract frame")??
        .into_data()
        .map_err(|e| anyhow::anyhow!("{:?}", e))?;

    Ok(
        build.body(lambda_http::Body::Binary(Vec::from(
            bytes,
        )))?
    )
}

pub fn init_http() -> Arc<LambdaHttp> {
    Arc::new(LambdaHttp::init())
}
