use anyhow::{bail, Result};
use reqwest::Request;

use crate::http::{HttpClient, Response};

use super::protobuf::ProtobufOperation;

pub async fn execute_operation_request(
  client: &dyn HttpClient,
  operation: &ProtobufOperation,
  mut request: Request,
) -> Result<Response> {
  let body = if let Some(body) = request.body() {
    operation.convert_input(body.as_bytes().unwrap_or_default())?
  } else {
    Default::default()
  };
  request.body_mut().replace(body.into());
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
