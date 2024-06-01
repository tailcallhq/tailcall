use std::sync::Arc;

use anyhow::Result;
use hyper::body::Bytes;
use lambda_http::RequestExt;
use reqwest::Client;
use tailcall::core::http::Response;
use tailcall::core::HttpIO;

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
        let response = self
            .client
            .execute(request)
            .await?
            .error_for_status()
            .map_err(|err| err.without_url())?;
        let res = Response::from_reqwest(response).await?;
        tracing::info!("{} {}", req_str, res.status.as_u16());
        Ok(res)
    }
}

pub fn to_request(req: lambda_http::Request) -> anyhow::Result<hyper::Request<hyper::Body>> {
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

    // Re-construct real URL from parameters
    let url = format!(
        "{}://{}/{}",
        req.uri().scheme_str().unwrap_or("http"),
        req.uri()
            .host()
            .ok_or(anyhow::anyhow!("Invalid request host"))?,
        req.path_parameters()
            .all("proxy")
            .unwrap_or(Vec::with_capacity(0))
            .join("/")
    );

    let mut req2 = hyper::Request::builder().method(method).uri(url);

    for (k, v) in req.headers() {
        let key: hyper::http::header::HeaderName = k.as_str().parse()?;
        let value = hyper::http::header::HeaderValue::from_bytes(v.as_bytes())?;
        req2 = req2.header(key, value);
    }

    Ok(req2.body(hyper::Body::from(req.body().to_vec()))?)
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

#[cfg(test)]
mod tests {
    use lambda_http::http::{Method, Request, StatusCode, Uri};
    use lambda_http::Body;

    use super::*;

    #[tokio::test]
    async fn test_to_request() {
        let req = Request::builder()
            .method(Method::GET)
            .uri(Uri::from_static("http://example.com"))
            .header("content-type", "application/json")
            .header("x-custom-header", "custom-value")
            .body(Body::from("Hello, world!"))
            .unwrap();
        let hyper_req = to_request(req).unwrap();
        assert_eq!(hyper_req.method(), hyper::Method::GET);
        assert_eq!(hyper_req.uri(), "http://example.com/");
        assert_eq!(
            hyper_req.headers().get("content-type").unwrap(),
            "application/json"
        );
        assert_eq!(
            hyper_req.headers().get("x-custom-header").unwrap(),
            "custom-value"
        );
    }

    #[tokio::test]
    async fn test_to_response() {
        let res = hyper::Response::builder()
            .status(200)
            .header("content-type", "application/json")
            .header("x-custom-header", "custom-value")
            .body(hyper::Body::from("Hello, world!"))
            .unwrap();
        let lambda_res = to_response(res).await.unwrap();
        assert_eq!(lambda_res.status(), StatusCode::OK);
        assert_eq!(
            lambda_res.headers().get("content-type").unwrap(),
            "application/json"
        );
        assert_eq!(
            lambda_res.headers().get("x-custom-header").unwrap(),
            "custom-value"
        );
    }
}
