extern crate core;

use std::path::Path;
use std::str::FromStr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use anyhow::anyhow;
use hyper::body::Bytes;
use reqwest::header::{HeaderName, HeaderValue};
use tailcall::core::http::{HttpFilter, Response};
use tailcall::core::HttpIO;

use super::runtime::{ExecutionMock, ExecutionSpec};

#[derive(Clone, Debug)]
pub struct Http {
    mocks: Vec<ExecutionMock>,
    spec_path: String,
}

impl Http {
    pub fn new(spec: &ExecutionSpec) -> Self {
        let mocks = spec
            .mock
            .as_ref()
            .map(|mocks| {
                mocks
                    .iter()
                    .map(|mock| ExecutionMock {
                        mock: mock.clone(),
                        actual_hits: Arc::new(AtomicUsize::default()),
                    })
                    .collect()
            })
            .unwrap_or_default();

        let spec_path = spec
            .path
            .strip_prefix(std::env::current_dir().unwrap())
            .unwrap_or(&spec.path)
            .to_string_lossy()
            .into_owned();

        Http { mocks, spec_path }
    }

    pub fn test_hits(&self, path: impl AsRef<Path>) {
        for mock in &self.mocks {
            mock.test_hits(path.as_ref());
        }
    }
}

#[async_trait::async_trait]
impl HttpIO for Http {
    async fn execute(&self, req: reqwest::Request) -> anyhow::Result<Response<Bytes>> {
        // Try to find a matching mock for the incoming request.
        let execution_mock = self
            .mocks
            .iter()
            .find(|mock| {
                let mock_req = &mock.mock.request;
                let method_match = req.method() == mock_req.0.method.clone().to_hyper();
                let url_match = req.url().as_str() == mock_req.0.url.clone().as_str();
                let body_match = mock_req
                    .0
                    .body
                    .as_ref()
                    .map(|body| {
                        let mock_body = body.to_bytes();

                        req.body()
                            .and_then(|body| body.as_bytes().map(|req_body| req_body == mock_body))
                            .unwrap_or(false)
                    })
                    .unwrap_or(true);

                let headers_match = req
                    .headers()
                    .iter()
                    .filter(|(key, _)| *key != "content-type")
                    .all(|(key, value)| {
                        let header_name = key.to_string();

                        let header_value = value.to_str().unwrap();
                        let mock_header_value = "".to_string();
                        let mock_header_value = mock_req
                            .0
                            .headers
                            .get(&header_name)
                            .unwrap_or(&mock_header_value);
                        header_value == mock_header_value
                    });
                method_match && url_match && headers_match && body_match
            })
            .ok_or(anyhow!(
                "No mock found for request: {:?} {} in {}",
                req.method(),
                req.url(),
                self.spec_path
            ))?;

        execution_mock.actual_hits.fetch_add(1, Ordering::Relaxed);

        // Clone the response from the mock to avoid borrowing issues.
        let mock_response = execution_mock.mock.response.clone();

        // Build the response with the status code from the mock.
        let status_code = reqwest::StatusCode::from_u16(mock_response.0.status)?;

        if status_code.is_client_error() || status_code.is_server_error() {
            return Err(anyhow::format_err!("Status code error"));
        }

        let mut response = Response { status: status_code, ..Default::default() };

        // Insert headers from the mock into the response.
        for (key, value) in mock_response.0.headers {
            let header_name = HeaderName::from_str(&key)?;
            let header_value = HeaderValue::from_str(&value)?;
            response.headers.insert(header_name, header_value);
        }

        // Special Handling for GRPC
        if let Some(body) = mock_response.0.body {
            response.body = Bytes::from(body.to_bytes());
        }

        Ok(response)
    }
}
