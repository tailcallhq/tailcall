use anyhow::Result;
use derive_setters::Setters;

use crate::grpc::protobuf::ProtobufOperation;

#[derive(Clone, Debug, Default, Setters)]
pub struct Response<Body: Default + Clone> {
  pub status: reqwest::StatusCode,
  pub headers: reqwest::header::HeaderMap,
  pub body: Body,
}

impl Response<Vec<u8>> {
  pub async fn from_reqwest(resp: reqwest::Response) -> Result<Response<Vec<u8>>> {
    let status = resp.status();
    let headers = resp.headers().to_owned();
    let body = resp.bytes().await?.to_vec();
    Ok(Response { status, headers, body })
  }

  pub fn to_json(self) -> Result<Response<async_graphql::Value>> {
    let mut resp = Response::default();
    let body = serde_json::from_slice::<async_graphql::Value>(&self.body)?;
    resp.body = body;
    resp.status = self.status;
    resp.headers = self.headers;
    Ok(resp)
  }

  pub fn to_grpc_value(self, operation: &ProtobufOperation) -> Result<Response<async_graphql::Value>> {
    let mut resp = Response::default();
    let body = operation.convert_output(&self.body)?;
    resp.body = body;
    resp.status = self.status;
    resp.headers = self.headers;
    Ok(resp)
  }

  pub fn to_resp_string(self) -> Result<Response<String>> {
    Ok(Response::<String> { body: String::from_utf8(self.body)?, status: self.status, headers: self.headers })
  }
}
