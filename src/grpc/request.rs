use anyhow::{bail, Result};
use reqwest::Request;

use super::protobuf::ProtobufOperation;
use crate::http::{HttpClient, Response};

pub async fn execute_grpc_request(
  client: &dyn HttpClient,
  operation: &ProtobufOperation,
  request: Request,
) -> Result<Response> {
  let response = client.execute_raw(request).await?;
  let status = response.status();
  let headers = response.headers().to_owned();

  if status.is_success() {
    let bytes = response.bytes().await?;
    let body = operation.convert_output(&bytes)?;

    return Ok(Response { status, headers, body });
  }

  bail!("Failed to execute request")
}
