use anyhow::Result;
use async_graphql_value::{ConstValue, Name};
use derive_setters::Setters;
use hyper::body::Bytes;
use indexmap::IndexMap;
use prost::Message;
use serde::de::DeserializeOwned;
use tonic::Status;
use tonic_types::Status as GrpcStatus;

use crate::grpc::protobuf::{ProtobufMessage, ProtobufOperation};
use crate::lambda::EvaluationError;

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
        let body = operation.convert_output::<async_graphql::Value>(&self.body)?;
        resp.body = body;
        resp.status = self.status;
        resp.headers = self.headers;
        Ok(resp)
    }

    pub fn to_grpc_error(&self, operation: &ProtobufOperation) -> anyhow::Error {
        let grpc_status = match Status::from_header_map(&self.headers) {
            Some(status) => status,
            None => {
                return EvaluationError::IOException(
                    "Error while parsing upstream headers".to_owned(),
                )
                .into()
            }
        };
        let mut obj = IndexMap::new();
        let mut status_details = Vec::new();
        if !grpc_status.details().is_empty() {
            if let Ok(any_details) = GrpcStatus::decode(grpc_status.details()) {
                let status: GrpcStatus = any_details;

                obj.insert(Name::new("code"), ConstValue::Number(status.code.into()));
                obj.insert(
                    Name::new("message"),
                    ConstValue::String(status.message.clone()),
                );

                for detail in status.details {
                    let type_url = &detail.type_url;
                    let type_name = type_url.split('/').last().unwrap_or("");

                    if let Some(message_descriptor) = operation
                        .method
                        .parent_pool()
                        .get_message_by_name(type_name)
                    {
                        let descriptor = ProtobufMessage { message_descriptor };
                        if let Ok(decoded) = descriptor.decode(detail.value.as_slice()) {
                            status_details.push(decoded);
                        } else {
                            tracing::error!("Error while decoding google.rpc.Status details");
                        }
                    } else {
                        tracing::error!(
                            "Error while searching descriptor for message: {}",
                            type_name
                        );
                    }
                }
            } else {
                tracing::error!("Error while decoding gRPC status details");
            }
        }
        obj.insert(Name::new("details"), ConstValue::List(status_details));
        let error = EvaluationError::GRPCError {
            grpc_code: grpc_status.code() as i32,
            grpc_description: grpc_status.code().description().to_owned(),
            grpc_status_message: grpc_status.message().to_owned(),
            grpc_status_details: ConstValue::Object(obj),
        };

        anyhow::Error::new(error)
    }

    pub fn to_resp_string(self) -> Result<Response<String>> {
        Ok(Response::<String> {
            body: String::from_utf8(self.body.to_vec())?,
            status: self.status,
            headers: self.headers,
        })
    }
}
