use anyhow::{bail, Result};
use hyper::{HeaderMap, Method};
use reqwest::Request;
use url::Url;

use super::protobuf::ProtobufOperation;
use crate::grpc::protobuf::ProtobufMessage;
use crate::http::Response;
use crate::runtime::TargetRuntime;

static GRPC_STATUS: &str = "grpc-status";

pub fn create_grpc_request(url: Url, headers: HeaderMap, body: Vec<u8>) -> Request {
    let mut req = Request::new(Method::POST, url);
    req.headers_mut().extend(headers.clone());
    req.body_mut().replace(body.into());

    req
}

pub async fn execute_grpc_request(
    runtime: &TargetRuntime,
    operation: &ProtobufOperation,
    status_details: &Option<ProtobufMessage>,
    request: Request,
) -> Result<Response<async_graphql::Value>> {
    let response = runtime.http2_only.execute(request).await?;

    let grpc_status = response.headers.get(GRPC_STATUS);

    if response.status.is_success() && grpc_status.is_none() {
        return response.to_grpc_value(operation);
    } else if grpc_status.is_some() {
        return response.to_grpc_error(status_details);
    }
    bail!("Failed to execute request");
}
