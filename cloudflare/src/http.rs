use anyhow::{anyhow, Result};
use async_std::task::spawn_local;
use hyper::body::Bytes;
use reqwest::Client;
use tailcall::http::Response;
use tailcall::HttpIO;

use crate::to_anyhow;

#[derive(Clone)]
pub struct CloudflareHttp {
    client: Client,
}

impl Default for CloudflareHttp {
    fn default() -> Self {
        Self { client: Client::new() }
    }
}

impl CloudflareHttp {
    pub fn init() -> Self {
        let client = Client::new();
        Self { client }
    }
}

#[async_trait::async_trait]
impl HttpIO for CloudflareHttp {
    // HttpClientOptions are ignored in Cloudflare
    // This is because there is little control over the underlying HTTP client
    async fn execute(&self, request: reqwest::Request) -> Result<Response<Bytes>> {
        let client = self.client.clone();
        let method = request.method().clone();
        let url = request.url().clone();
        // TODO: remove spawn local
        let res = spawn_local(async move {
            let response = client.execute(request).await?.error_for_status()?;
            Response::from_reqwest(response).await
        })
        .await?;
        tracing::info!("{} {} {}", method, url, res.status.as_u16());
        Ok(res)
    }
}

pub async fn to_response(response: hyper::Response<hyper::Body>) -> Result<worker::Response> {
    let status = response.status().as_u16();
    let headers = response.headers().clone();
    let bytes = hyper::body::to_bytes(response).await?;
    let body = worker::ResponseBody::Body(bytes.to_vec());
    let mut w_response = worker::Response::from_body(body).map_err(to_anyhow)?;
    w_response = w_response.with_status(status);
    let mut_headers = w_response.headers_mut();
    for (name, value) in headers.iter() {
        let value = String::from_utf8(value.as_bytes().to_vec())?;
        mut_headers
            .append(name.as_str(), &value)
            .map_err(to_anyhow)?;
    }

    Ok(w_response)
}

pub fn to_method(method: worker::Method) -> Result<hyper::Method> {
    let method = &*method.to_string().to_uppercase();
    match method {
        "GET" => Ok(hyper::Method::GET),
        "POST" => Ok(hyper::Method::POST),
        "PUT" => Ok(hyper::Method::PUT),
        "DELETE" => Ok(hyper::Method::DELETE),
        "HEAD" => Ok(hyper::Method::HEAD),
        "OPTIONS" => Ok(hyper::Method::OPTIONS),
        "PATCH" => Ok(hyper::Method::PATCH),
        "CONNECT" => Ok(hyper::Method::CONNECT),
        "TRACE" => Ok(hyper::Method::TRACE),
        method => Err(anyhow!("Unsupported HTTP method: {}", method)),
    }
}

pub async fn to_request(mut req: worker::Request) -> Result<hyper::Request<hyper::Body>> {
    let body = req.text().await.map_err(to_anyhow)?;
    let method = req.method();
    let uri = req.url().map_err(to_anyhow)?.as_str().to_string();
    let headers = req.headers();
    let mut builder = hyper::Request::builder()
        .method(to_method(method)?)
        .uri(uri);
    for (k, v) in headers {
        builder = builder.header(k, v);
    }
    Ok(builder.body(hyper::body::Body::from(body))?)
}
