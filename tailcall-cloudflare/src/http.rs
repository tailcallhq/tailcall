use async_std::task::spawn_local;
use hyper::body::Bytes;
use reqwest::Client;
use tailcall::core::error::http;
use tailcall::core::http::Response;
use tailcall::core::HttpIO;

use super::Error;

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
    async fn execute(&self, request: reqwest::Request) -> Result<Response<Bytes>, http::Error> {
        let client = self.client.clone();
        let method = request.method().clone();
        let url = request.url().clone();
        // TODO: remove spawn local
        let res = spawn_local(async move {
            let response = client
                .execute(request)
                .await?
                .error_for_status()
                .map_err(|err| err.without_url())?;
            Response::from_reqwest(response).await
        })
        .await?;
        tracing::info!("{} {} {}", method, url, res.status.as_u16());
        Ok(res)
    }
}

pub async fn to_response(
    response: hyper::Response<hyper::Body>,
) -> Result<worker::Response, Error> {
    let status = response.status().as_u16();
    let headers = response.headers().clone();
    let bytes = hyper::body::to_bytes(response).await?;
    let body = worker::ResponseBody::Body(bytes.to_vec());
    let mut w_response = worker::Response::from_body(body)?;
    w_response = w_response.with_status(status);
    let mut_headers = w_response.headers_mut();
    for (name, value) in headers.iter() {
        let value = String::from_utf8(value.as_bytes().to_vec())?;
        mut_headers.append(name.as_str(), &value)?;
    }

    Ok(w_response)
}

pub fn to_method(method: worker::Method) -> Result<hyper::Method, Error> {
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
        method => Err(Error::UnsupportedHttpMethod(method.to_string())),
    }
}

pub async fn to_request(mut req: worker::Request) -> Result<hyper::Request<hyper::Body>, Error> {
    let body = req.text().await?;
    let method = req.method();
    let uri = req.url()?.as_str().to_string();
    let headers = req.headers();
    let mut builder = hyper::Request::builder()
        .method(to_method(method)?)
        .uri(uri);
    for (k, v) in headers {
        builder = builder.header(k, v);
    }
    Ok(builder.body(hyper::body::Body::from(body))?)
}
