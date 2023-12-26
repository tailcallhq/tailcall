use anyhow::{bail, Result};
use hyper::{HeaderMap, Method};
use reqwest::Request;
use url::Url;

use super::protobuf::ProtobufOperation;
use crate::http::{HttpClient, Response};

pub fn create_grpc_request(url: Url, headers: HeaderMap, body: Vec<u8>) -> Request {
  let mut req = Request::new(Method::POST, url);
  #[cfg(feature = "default")]
  set_version(req.version_mut());
  req.headers_mut().extend(headers.clone());
  req.body_mut().replace(body.into());

  req
}
#[cfg(feature = "default")]
fn set_version(version: &mut reqwest::Version) {
  *version = reqwest::Version::HTTP_2;
}

pub async fn execute_grpc_request(
  client: &dyn HttpClient,
  operation: &ProtobufOperation,
  request: Request,
) -> Result<Response> {
  let response = client.execute(request, Some(operation)).await?;
  if response.status.is_success() {
    return Ok(response);
  }
  bail!("Failed to execute request")
}
