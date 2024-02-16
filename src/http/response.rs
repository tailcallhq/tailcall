use anyhow::Result;
use derive_setters::Setters;
use hyper::body::Bytes;
use serde::de::DeserializeOwned;

use crate::grpc::protobuf::ProtobufOperation;

#[derive(Clone, Debug, Default, Setters)]
pub struct Response<Body> {
    pub status: reqwest::StatusCode,
    pub headers: reqwest::header::HeaderMap,
    pub body: Body,
}

impl Response<Bytes> {
    pub async fn from_reqwest(resp: reqwest::Response) -> Result<Self> {
        let status = resp.status();
        let headers = resp.headers().to_owned();
        let body = resp.bytes().await?;
        Ok(Response { status, headers, body })
    }

    pub fn empty() -> Self {
        Response {
            status: reqwest::StatusCode::OK,
            headers: reqwest::header::HeaderMap::default(),
            body: Bytes::new(),
        }
    }

    pub fn to_json<T: DeserializeOwned>(self) -> Result<Response<T>> {
        let body = serde_json::from_slice::<T>(&self.body)?;
        Ok(Response { status: self.status, headers: self.headers, body })
    }

    pub fn to_grpc_value(
        self,
        operation: &ProtobufOperation,
    ) -> Result<Response<async_graphql::Value>> {
        let mut resp = Response::default();
        let body = operation.convert_output(&self.body)?;
        resp.body = body;
        resp.status = self.status;
        resp.headers = self.headers;
        Ok(resp)
    }

    pub fn to_resp_string(self) -> Result<Response<String>> {
        Ok(Response::<String> {
            body: String::from_utf8(self.body.to_vec())?,
            status: self.status,
            headers: self.headers,
        })
    }
}
