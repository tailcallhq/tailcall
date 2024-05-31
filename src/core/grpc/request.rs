use anyhow::{bail, Result};
use hyper::{HeaderMap, Method};
use reqwest::Request;
use url::Url;

use super::protobuf::ProtobufOperation;
use crate::core::http::Response;
use crate::core::runtime::TargetRuntime;

pub static GRPC_STATUS: &str = "grpc-status";

pub fn create_grpc_request(url: Url, headers: HeaderMap, body: Vec<u8>) -> Request {
    let mut req = Request::new(Method::POST, url);
    req.headers_mut().extend(headers.clone());
    req.body_mut().replace(body.into());

    req
}

pub async fn execute_grpc_request(
    runtime: &TargetRuntime,
    operation: &ProtobufOperation,
    request: Request,
) -> Result<Response<async_graphql::Value>> {
    let response = runtime.http2_only.execute(request).await?;

    let grpc_status = response
        .headers
        .get(GRPC_STATUS)
        .and_then(|header_value| header_value.to_str().ok());

    if response.status.is_success() {
        return if grpc_status.is_none() || grpc_status == Some("0") {
            response.to_grpc_value(operation)
        } else {
            Err(response.to_grpc_error(operation))
        };
    }
    bail!("Failed to execute request");
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use anyhow::Result;
    use async_trait::async_trait;
    use hyper::body::Bytes;
    use reqwest::header::HeaderMap;
    use reqwest::{Method, Request, StatusCode};
    use serde_json::json;
    use tailcall_fixtures::protobuf;
    use tonic::{Code, Status};

    use crate::core::blueprint::GrpcMethod;
    use crate::core::grpc::protobuf::{ProtobufOperation, ProtobufSet};
    use crate::core::grpc::request::execute_grpc_request;
    use crate::core::http::Response;
    use crate::core::ir::EvaluationError;
    use crate::core::runtime::TargetRuntime;
    use crate::core::HttpIO;

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
            let error = Bytes::from_static(b"\x08\x03\x12\x0Derror message\x1A\x3E\x0A+type.googleapis.com/greetings.ErrValidation\x12\x0F\x0A\x0Derror details");

            match self.scenario {
                TestScenario::SuccessWithoutGrpcStatus => {
                    Ok(Response { status: StatusCode::OK, headers, body: message })
                }
                TestScenario::SuccessWithOkGrpcStatus => {
                    let status = Status::ok("");
                    status.add_header(&mut headers)?;
                    Ok(Response { status: StatusCode::OK, headers, body: message })
                }
                TestScenario::SuccessWithErrorGrpcStatus => {
                    let status =
                        Status::with_details(Code::InvalidArgument, "description message", error);
                    status.add_header(&mut headers)?;
                    Ok(Response { status: StatusCode::OK, headers, body: Bytes::default() })
                }
                TestScenario::Error => Ok(Response {
                    status: StatusCode::NOT_FOUND,
                    headers,
                    body: Bytes::default(),
                }),
            }
        }
    }
    async fn prepare_args(
        test_http: TestHttp,
    ) -> Result<(TargetRuntime, ProtobufOperation, Request)> {
        let mut runtime = crate::core::runtime::test::init(None);
        runtime.http2_only = Arc::new(test_http);

        let file_descriptor_set =
            protox::compile([protobuf::GREETINGS, protobuf::ERRORS], [protobuf::SELF]);
        let grpc_method = GrpcMethod::try_from("greetings.Greeter.SayHello").unwrap();
        let file = ProtobufSet::from_proto_file(file_descriptor_set.unwrap())?;
        let service = file.find_service(&grpc_method)?;
        let operation = service.find_operation(&grpc_method)?;

        let request = Request::new(Method::POST, "http://example.com".parse().unwrap());
        Ok((runtime, operation, request))
    }

    #[tokio::test]
    async fn test_grpc_request_success_without_grpc_status() -> Result<()> {
        let test_http = TestHttp { scenario: TestScenario::SuccessWithoutGrpcStatus };
        let (runtime, operation, request) = prepare_args(test_http).await?;

        let result = execute_grpc_request(&runtime, &operation, request).await;

        assert!(
            result.is_ok(),
            "Expected a successful response without grpc-status"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_grpc_request_success_with_ok_grpc_status() -> Result<()> {
        let test_http = TestHttp { scenario: TestScenario::SuccessWithOkGrpcStatus };
        let (runtime, operation, request) = prepare_args(test_http).await?;

        let result = execute_grpc_request(&runtime, &operation, request).await;

        assert!(
            result.is_ok(),
            "Expected a successful response with '0' (Ok) grpc-status"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_grpc_request_success_with_error_grpc_status() -> Result<()> {
        let test_http = TestHttp { scenario: TestScenario::SuccessWithErrorGrpcStatus };
        let (runtime, operation, request) = prepare_args(test_http).await?;

        let result = execute_grpc_request(&runtime, &operation, request).await;

        assert!(
            result.is_err(),
            "Expected an error response due to grpc-status"
        );

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
                            "code": 3,
                            "message": "error message",
                            "details": [{
                                "error": "error details",
                            }]
                        })
                    );
                }
                _ => panic!("Expected GRPCError"),
            }
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_grpc_request_error() -> Result<()> {
        let test_http = TestHttp { scenario: TestScenario::Error };
        let (runtime, operation, request) = prepare_args(test_http).await?;

        let result = execute_grpc_request(&runtime, &operation, request).await;

        assert!(result.is_err(), "Expected error");
        assert_eq!(result.unwrap_err().to_string(), "Failed to execute request");

        Ok(())
    }
}
