use anyhow::Result;
use derive_setters::Setters;

use crate::grpc::protobuf::ProtobufOperation;

#[derive(Clone, Debug, Default, Setters)]
pub struct Response<Body: Default + Clone> {
  pub status: reqwest::StatusCode,
  pub headers: reqwest::header::HeaderMap,
  pub body: Body,
}

impl Response<String> {
  pub async fn from_response_to_string(resp: reqwest::Response) -> Result<Response<String>> {
    let status = resp.status();
    let headers = resp.headers().to_owned();
    let body = resp.text().await?;
    Ok(Response { status, headers, body })
  }

}

impl Response<async_graphql::Value> {
  pub async fn from_response_to_val(resp: reqwest::Response) -> Result<Response<async_graphql::Value>> {
    let status = resp.status();
    let headers = resp.headers().to_owned();
    let body = resp.bytes().await?.to_vec();
    let body = serde_json::from_slice(&body)?;
    Ok(Response { status, headers, body })
  }
}

impl Response<Vec<u8>> {
  pub async fn from_response_to_vec(resp: reqwest::Response) -> Result<Response<Vec<u8>>> {
    let status = resp.status();
    let headers = resp.headers().to_owned();
    let body = resp.bytes().await?.to_vec();
    Ok(Response { status, headers, body })
  }

  pub fn to_value(self, operation: Option<&ProtobufOperation>) -> Result<Response<async_graphql::Value>> {
    let mut resp = Response::default();
    let body = match operation {
      None => serde_json::from_slice::<async_graphql::Value>(&self.body)?,
      Some(operation) => operation.convert_output(&self.body)?,
    };
    resp.body = body;
    resp.status = self.status;
    resp.headers = self.headers;
    Ok(resp)
  }

  pub fn to_resp_string(self) -> Result<Response<String>> {
    let mut resp = Response::default();
    resp.body = String::from_utf8(self.body)?;
    resp.status = self.status;
    resp.headers = self.headers;
    Ok(resp)
  }
}
