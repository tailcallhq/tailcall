use anyhow::Result;
use derive_setters::Setters;

use crate::grpc::protobuf::ProtobufOperation;

#[derive(Clone, Debug, Default, Setters)]
pub struct Response {
  pub status: reqwest::StatusCode,
  pub headers: reqwest::header::HeaderMap,
  pub body: async_graphql::Value,
}

impl Response {
  pub async fn from_response(resp: reqwest::Response, operation: Option<ProtobufOperation>) -> Result<Self> {
    let status = resp.status();
    let headers = resp.headers().to_owned();
    let body = resp.bytes().await?;
    let body = match operation {
      None => serde_json::from_slice(&body)?,
      Some(operation) => operation.convert_output(&body)?,
    };
    Ok(Response { status, headers, body })
  }
}
