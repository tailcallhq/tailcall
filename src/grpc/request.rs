use anyhow::{bail, Result};
use hyper::{HeaderMap, Method};
use reqwest::Request;
use url::Url;

use super::protobuf::ProtobufOperation;
use crate::grpc::protobuf::ProtobufMessage;
use crate::http::Response;
use crate::runtime::TargetRuntime;

pub static GRPC_STATUS: &str = "grpc-status";
pub static GRPC_MESSAGE: &str = "grpc-message";
pub static GRPC_STATUS_DETAILS: &str = "grpc-status-details-bin";

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

    let grpc_status = response.headers.get(GRPC_STATUS)
        .and_then(|header_value| header_value.to_str().ok());

    if response.status.is_success() {
        return if grpc_status.is_none() || grpc_status == Some("0") {
            response.to_grpc_value(operation)
        } else {
            response.to_grpc_error(status_details)
        }
    }
    bail!("Failed to execute request");
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use reqwest::header::HeaderMap;
    use crate::grpc::request::{GRPC_MESSAGE, GRPC_STATUS, GRPC_STATUS_DETAILS};
    use crate::http::Response;
    use crate::HttpIO;
    use hyper::body::Bytes;
    use anyhow::Result;
    use async_trait::async_trait;
    use base64::Engine;
    use base64::prelude::BASE64_STANDARD_NO_PAD;
    use hyper::header::HeaderValue;
    use reqwest::{Method, Request, StatusCode};
    use serde_json::json;
    use tonic::Code;
    use crate::blueprint::{GrpcMessage, GrpcMethod};
    use crate::grpc::execute_grpc_request;
    use crate::grpc::protobuf::{ProtobufMessage, ProtobufOperation, ProtobufSet};
    use crate::grpc::protobuf::tests::get_proto_file;
    use crate::lambda::EvaluationError;
    use crate::runtime::TargetRuntime;

    enum TestScenario {
        SuccessWithoutGrpcStatus,
        SuccessWithOkGrpcStatus,
        SuccessWithErrorGrpcStatus,
        Error,
    }

    struct TestHttp {
        scenario: TestScenario,
    }

    #[async_trait]
    impl HttpIO for TestHttp {
        async fn execute(&self, _request: Request) -> Result<Response<Bytes>> {
            let mut headers = HeaderMap::new();
            let message = Bytes::from_static(b"\0\0\0\0\x0e\n\x0ctest message");
            let error = BASE64_STANDARD_NO_PAD.encode(b"\x0a\x0derror message");

            match self.scenario {
                TestScenario::SuccessWithoutGrpcStatus => {
                    Ok(Response {
                        status: StatusCode::OK,
                        headers,
                        body: message,
                    })
                },
                TestScenario::SuccessWithOkGrpcStatus => {
                    headers.insert(GRPC_STATUS, HeaderValue::from_static("0"));
                    Ok(Response {
                        status: StatusCode::OK,
                        headers,
                        body: message,
                    })
                },
                TestScenario::SuccessWithErrorGrpcStatus => {
                    headers.insert(GRPC_STATUS, HeaderValue::from_static("3"));
                    headers.insert(GRPC_MESSAGE, HeaderValue::from_static("description message"));
                    headers.insert(GRPC_STATUS_DETAILS, HeaderValue::try_from(error).unwrap());
                    Ok(Response {
                        status: StatusCode::OK,
                        headers,
                        body: Bytes::default(),
                    })
                },
                TestScenario::Error => {
                    Ok(Response {
                        status: StatusCode::NOT_FOUND,
                        headers,
                        body: Bytes::default(),
                    })
                },
            }
        }
    }
    async fn prepare_args(test_http: TestHttp) -> Result<(TargetRuntime, ProtobufOperation, Option<ProtobufMessage>, Request)> {
        let mut runtime = crate::runtime::test::init(None);
        runtime.http2_only = Arc::new(test_http);

        let grpc_method = GrpcMethod::try_from("greetings.Greeter.SayHello").unwrap();
        let file = ProtobufSet::from_proto_file(get_proto_file("greetings.proto").await?)?;
        let service = file.find_service(&grpc_method)?;
        let operation = service.find_operation(&grpc_method)?;

        let grpc_message = GrpcMessage::try_from("greetings.ErrValidation").unwrap();
        let file = ProtobufSet::from_proto_file(get_proto_file("errors.proto").await?)?;
        let status_details = file.find_message(&grpc_message).ok();

        let request = Request::new(Method::POST, "http://example.com".parse().unwrap());
        Ok((runtime, operation, status_details, request))
    }

    #[tokio::test]
    async fn test_grpc_request_success_without_grpc_status() -> Result<()> {
        let test_http = TestHttp {
            scenario: TestScenario::SuccessWithoutGrpcStatus,
        };
        let (runtime, operation, status_details, request) = prepare_args(test_http).await?;

        let result = execute_grpc_request(&runtime, &operation, &status_details, request).await;


        assert!(result.is_ok(), "Expected a successful response without grpc-status");
        Ok(())
    }

    #[tokio::test]
    async fn test_grpc_request_success_with_ok_grpc_status() -> Result<()> {
        let test_http = TestHttp {
            scenario: TestScenario::SuccessWithOkGrpcStatus,
        };
        let (runtime, operation, status_details, request) = prepare_args(test_http).await?;

        let result = execute_grpc_request(&runtime, &operation, &status_details, request).await;

        assert!(result.is_ok(), "Expected a successful response with '0' (Ok) grpc-status");
        Ok(())
    }

    #[tokio::test]
    async fn test_grpc_request_success_with_error_grpc_status() -> Result<()> {
        let test_http = TestHttp {
            scenario: TestScenario::SuccessWithErrorGrpcStatus,
        };
        let (runtime, operation, status_details, request) = prepare_args(test_http).await?;

        let result = execute_grpc_request(&runtime, &operation, &status_details, request).await;

        assert!(result.is_err(), "Expected an error response due to grpc-status");

        if let Err(err) = result {
            match err.downcast_ref::<EvaluationError>() {
                Some(EvaluationError::GRPCError {
                         grpc_code,
                         grpc_description,
                         grpc_status_message,
                         grpc_status_details,
                     }) => {
                    let code = Code::InvalidArgument;
                    assert_eq!(*grpc_code, code as i32);
                    assert_eq!(*grpc_description, code.description());
                    assert_eq!(*grpc_status_message, "description message");
                    assert_eq!(
                        serde_json::to_value(grpc_status_details)?,
                        json!({
                          "error": "error message"
                        })
                    );
                },
                _ => panic!("Expected GRPCError"),
            }
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_grpc_request_error() -> Result<()> {
        let test_http = TestHttp {
            scenario: TestScenario::Error,
        };
        let (runtime, operation, status_details, request) = prepare_args(test_http).await?;

        let result = execute_grpc_request(&runtime, &operation, &status_details, request).await;

        assert!(result.is_err(), "Expected error");
        assert_eq!(result.unwrap_err().to_string(), "Failed to execute request");

        Ok(())
    }
}
