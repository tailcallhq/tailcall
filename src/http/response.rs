use anyhow::Result;
use derive_setters::Setters;
use hyper::body::Bytes;
use tonic::Code;

use crate::grpc::protobuf::{ProtobufMessage, ProtobufOperation};
use crate::lambda::EvaluationError;

pub(crate) static GRPC_STATUS: &str = "grpc-status";
pub(crate) static GRPC_MESSAGE: &str = "grpc-message";
pub(crate) static GRPC_STATUS_DETAILS: &str = "grpc-status-details-bin";

#[derive(Clone, Debug, Default, Setters)]
pub struct Response<Body: Default + Clone> {
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

    pub fn to_json(self) -> Result<Response<async_graphql::Value>> {
        let mut resp = Response::default();
        let body = serde_json::from_slice::<async_graphql::Value>(&self.body)?;
        resp.body = body;
        resp.status = self.status;
        resp.headers = self.headers;
        Ok(resp)
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

    pub fn get_header_value(&self, header_name: &str) -> Option<String> {
        self.headers
            .get(header_name)
            .and_then(|header_value| header_value.to_str().ok())
            .map(|s| s.to_string())
    }

    pub fn to_grpc_error(
        self,
        status_details: &Option<ProtobufMessage>,
    ) -> Result<Response<async_graphql::Value>> {
        let grpc_status = self.get_header_value(GRPC_STATUS);
        let grpc_message = self.get_header_value(GRPC_MESSAGE);

        let grpc_code = Code::from(
            grpc_status
                .and_then(|v| v.parse::<i32>().ok())
                .unwrap_or(-1),
        );

        let details = self
            .get_header_value(GRPC_STATUS_DETAILS)
            .and_then(|d| {
                let status = status_details.as_ref()?.decode(d.as_bytes());
                if let Err(ref error) = status {
                    tracing::error!("Error while decoding status_details: {}", error);
                }
                status.ok()
            });

        let error = EvaluationError::GRPCError {
            grpc_code: grpc_code as i32,
            grpc_description: grpc_code.description().to_string(),
            grpc_status_message: grpc_message.unwrap_or_default(),
            grpc_status_details: details.unwrap_or_default(),
        };
        Err(error.into())
    }

    pub fn to_resp_string(self) -> Result<Response<String>> {
        Ok(Response::<String> {
            body: String::from_utf8(self.body.to_vec())?,
            status: self.status,
            headers: self.headers,
        })
    }
}

#[cfg(test)]
pub mod tests {
    use anyhow::Result;
    use hyper::http::HeaderValue;
    use crate::http::Response;

    #[tokio::test]
    async fn test_get_header_value_found() -> Result<()> {
        let mut resp = Response::default();
        let key = "header";
        resp.headers.insert(key, HeaderValue::from_static("value"));
        let val = resp.get_header_value(key);

        assert!(val.is_some());
        Ok(())
    }

    #[tokio::test]
    async fn test_get_header_value_not_found() -> Result<()> {
        let mut resp = Response::default();
        let key = "header";
        let val = resp.get_header_value(key);

        assert!(val.is_none());
        Ok(())
    }
}
