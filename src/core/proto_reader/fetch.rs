use std::str::FromStr;

use anyhow::{Context, Result};
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use http::header::HeaderName;
use nom::AsBytes;
use prost::Message;
use prost_reflect::prost_types::{FileDescriptorProto, FileDescriptorSet};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::core::blueprint::GrpcMethod;
use crate::core::config::{ConfigReaderContext, KeyValue};
use crate::core::grpc::protobuf::ProtobufSet;
use crate::core::grpc::request_template::RequestBody;
use crate::core::grpc::RequestTemplate;
use crate::core::mustache::Mustache;
use crate::core::runtime::TargetRuntime;

///
/// Loading reflection proto
/// https://github.com/grpc/grpc/blob/master/src/proto/grpc/reflection/v1alpha/reflection.proto
const REFLECTION_PROTO: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/core/proto_reader/proto/reflection.proto"
));

/// This function is just used for better exception handling
fn get_protobuf_set() -> Result<ProtobufSet> {
    let descriptor = protox_parse::parse("reflection", REFLECTION_PROTO)?;
    let mut descriptor_set = FileDescriptorSet::default();
    descriptor_set.file.push(descriptor);
    ProtobufSet::from_proto_file(descriptor_set)
}

#[derive(Debug, Serialize, Deserialize)]
struct Service {
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ListServicesResponse {
    service: Vec<Service>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FileDescriptorProtoResponse {
    file_descriptor_proto: Vec<String>,
}

impl FileDescriptorProtoResponse {
    fn get(self) -> Result<Vec<u8>> {
        let file_descriptor_proto = self
            .file_descriptor_proto
            .first()
            .context("Received empty fileDescriptorProto")?;

        BASE64_STANDARD
            .decode(file_descriptor_proto)
            .context("Failed to decode fileDescriptorProto from BASE64")
    }
}

/// Used for serializing all kinds of GRPC Reflection responses
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReflectionResponse {
    list_services_response: Option<ListServicesResponse>,
    file_descriptor_response: Option<FileDescriptorProtoResponse>,
}

pub struct GrpcReflection {
    server_reflection_method: GrpcMethod,
    url: String,
    headers: Option<Vec<KeyValue>>,
    target_runtime: TargetRuntime,
}

impl GrpcReflection {
    pub fn new<T: AsRef<str>>(
        url: T,
        headers: Option<Vec<KeyValue>>,
        target_runtime: TargetRuntime,
    ) -> Self {
        let server_reflection_method = GrpcMethod {
            package: "grpc.reflection.v1alpha".to_string(),
            service: "ServerReflection".to_string(),
            name: "ServerReflectionInfo".to_string(),
        };
        Self {
            server_reflection_method,
            url: url.as_ref().to_string(),
            headers,
            target_runtime,
        }
    }
    /// Makes `ListService` request to the grpc reflection server
    pub async fn list_all_files(&self) -> Result<Vec<String>> {
        // Extracting names from services
        let methods: Vec<String> = self
            .execute(json!({"list_services": ""}))
            .await?
            .list_services_response
            .context("Couldn't find definitions for service ServerReflection")?
            .service
            .iter()
            .map(|s| s.name.clone())
            .collect();

        Ok(methods)
    }

    /// Makes `Get Service` request to the grpc reflection server
    pub async fn get_by_service(&self, service: &str) -> Result<FileDescriptorProto> {
        let resp = self
            .execute(json!({"file_containing_symbol": service}))
            .await?;

        request_proto(resp)
    }

    /// Makes `Get File` request to grpc reflection server
    pub async fn get_file(&self, file_path: &str) -> Result<FileDescriptorProto> {
        let resp = self.execute(json!({"file_by_filename": file_path})).await?;

        request_proto(resp)
    }

    async fn execute(&self, body: serde_json::Value) -> Result<ReflectionResponse> {
        let server_reflection_method = &self.server_reflection_method;
        let protobuf_set = get_protobuf_set()?;
        let reflection_service = protobuf_set.find_service(server_reflection_method)?;
        let operation = reflection_service.find_operation(server_reflection_method)?;
        let mut url: url::Url = self.url.parse()?;
        url.set_path(
            format!(
                "{}.{}/{}",
                server_reflection_method.package,
                server_reflection_method.service,
                server_reflection_method.name
            )
            .as_str(),
        );

        let mut headers = vec![];
        if let Some(custom_headers) = &self.headers {
            for header in custom_headers {
                headers.push((
                    HeaderName::from_str(&header.key)?,
                    Mustache::parse(header.value.as_str()),
                ));
            }
        }
        headers.push((
            HeaderName::from_static("content-type"),
            Mustache::parse("application/grpc+proto"),
        ));
        let body_ = Some(RequestBody {
            mustache: Some(Mustache::parse(body.to_string().as_str())),
            value: Default::default(),
        });
        let req_template = RequestTemplate {
            url: Mustache::parse(url.as_str()),
            headers,
            body: body_,
            operation: operation.clone(),
            operation_type: Default::default(),
        };

        let ctx = ConfigReaderContext::new(&self.target_runtime);

        let req = req_template.render(&ctx)?.to_request()?;
        let resp = self.target_runtime.http2_only.execute(req).await?;
        let body = resp.body.as_bytes();

        let response: ReflectionResponse = operation.convert_output(body)?;
        Ok(response)
    }
}

/// For extracting `FileDescriptorProto` from `CustomResponse`
fn request_proto(response: ReflectionResponse) -> Result<FileDescriptorProto> {
    let file_descriptor_resp = response
        .file_descriptor_response
        .context("Expected fileDescriptorResponse but found none")?;
    let file_descriptor_proto =
        FileDescriptorProto::decode(file_descriptor_resp.get()?.as_bytes())?;

    Ok(file_descriptor_proto)
}

#[cfg(test)]
mod grpc_fetch {
    use std::path::PathBuf;

    use anyhow::Result;

    use super::*;

    fn get_fake_descriptor() -> Vec<u8> {
        let mut path = PathBuf::from(file!());
        path.pop();
        path.push("fixtures/descriptor_b64.txt");

        let bytes = std::fs::read(path).unwrap();

        BASE64_STANDARD.decode(bytes).unwrap()
    }

    fn get_fake_resp() -> Vec<u8> {
        let mut path = PathBuf::from(file!());
        path.pop();
        path.push("fixtures/response_b64.txt");

        let bytes = std::fs::read(path).unwrap();

        BASE64_STANDARD.decode(bytes).unwrap()
    }

    fn get_dto_file_descriptor() -> Vec<u8> {
        let mut path = PathBuf::from(file!());
        path.pop();
        path.push("fixtures/dto_b64.txt");

        let bytes = std::fs::read(path).unwrap();

        BASE64_STANDARD.decode(bytes).unwrap()
    }

    fn start_mock_server() -> httpmock::MockServer {
        httpmock::MockServer::start()
    }

    #[tokio::test]
    async fn test_resp_service() -> Result<()> {
        let server = start_mock_server();

        let http_reflection_file_mock = server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .path("/grpc.reflection.v1alpha.ServerReflection/ServerReflectionInfo")
                .body("\0\0\0\0\x12\"\x10news.NewsService");
            then.status(200).body(get_fake_descriptor());
        });

        let grpc_reflection = GrpcReflection::new(
            format!("http://localhost:{}", server.port()),
            None,
            crate::core::runtime::test::init(None),
        );

        let runtime = crate::core::runtime::test::init(None);
        let resp = grpc_reflection.get_by_service("news.NewsService").await?;

        let content = runtime.file.read(tailcall_fixtures::protobuf::NEWS).await?;
        let expected = protox_parse::parse("news.proto", &content)?;

        assert_eq!(expected.name(), resp.name());

        http_reflection_file_mock.assert();
        Ok(())
    }

    #[tokio::test]
    async fn test_dto_file() -> Result<()> {
        let server = start_mock_server();

        let http_reflection_file_mock = server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .path("/grpc.reflection.v1alpha.ServerReflection/ServerReflectionInfo")
                .body("\0\0\0\0\u{10}\u{1a}\u{0e}news_dto.proto");
            then.status(200).body(get_dto_file_descriptor());
        });

        let grpc_reflection = GrpcReflection::new(
            format!("http://localhost:{}", server.port()),
            None,
            crate::core::runtime::test::init(None),
        );

        let runtime = crate::core::runtime::test::init(None);
        let resp = grpc_reflection.get_file("news_dto.proto").await?;

        let content = runtime
            .file
            .read(tailcall_fixtures::protobuf::NEWS_DTO)
            .await?;
        let expected = protox_parse::parse("news_dto.proto", &content)?;

        assert_eq!(expected.name(), resp.name());

        http_reflection_file_mock.assert();
        Ok(())
    }

    #[tokio::test]
    async fn test_resp_list_all() -> Result<()> {
        let server = start_mock_server();

        let http_reflection_list_all = server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .path("/grpc.reflection.v1alpha.ServerReflection/ServerReflectionInfo")
                .body("\0\0\0\0\x02:\0");
            then.status(200).body(get_fake_resp());
        });

        let runtime = crate::core::runtime::test::init(None);

        let grpc_reflection =
            GrpcReflection::new(format!("http://localhost:{}", server.port()), None, runtime);

        let resp = grpc_reflection.list_all_files().await?;

        assert_eq!(
            [
                "news.NewsService".to_string(),
                "grpc.reflection.v1alpha.ServerReflection".to_string()
            ]
            .to_vec(),
            resp
        );

        http_reflection_list_all.assert();

        Ok(())
    }

    #[tokio::test]
    async fn test_list_all_files_empty_response() -> Result<()> {
        let server = start_mock_server();

        let http_reflection_list_all_empty = server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .path("/grpc.reflection.v1alpha.ServerReflection/ServerReflectionInfo")
                .body("\0\0\0\0\x02:\0");
            then.status(200).body("\0\0\0\0\x02:\0"); // Mock an empty response
        });

        let runtime = crate::core::runtime::test::init(None);

        let grpc_reflection =
            GrpcReflection::new(format!("http://localhost:{}", server.port()), None, runtime);

        let resp = grpc_reflection.list_all_files().await;

        assert_eq!(
            "Couldn't find definitions for service ServerReflection",
            resp.err().unwrap().to_string()
        );

        http_reflection_list_all_empty.assert();

        Ok(())
    }

    #[tokio::test]
    async fn test_get_by_service_not_found() -> Result<()> {
        let server = start_mock_server();

        let http_reflection_service_not_found = server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .path("/grpc.reflection.v1alpha.ServerReflection/ServerReflectionInfo");
            then.status(404); // Mock a 404 not found response
        });

        let runtime = crate::core::runtime::test::init(None);

        let grpc_reflection =
            GrpcReflection::new(format!("http://localhost:{}", server.port()), None, runtime);

        let result = grpc_reflection.get_by_service("nonexistent.Service").await;
        assert!(result.is_err());

        http_reflection_service_not_found.assert();

        Ok(())
    }

    #[tokio::test]
    async fn test_custom_headers_resp_list_all() -> Result<()> {
        let server = start_mock_server();

        let http_reflection_service_not_found = server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .path("/grpc.reflection.v1alpha.ServerReflection/ServerReflectionInfo")
                .header("authorization", "Bearer 123");
            then.status(200).body(get_fake_resp());
        });

        let runtime = crate::core::runtime::test::init(None);

        let grpc_reflection = GrpcReflection::new(
            format!("http://localhost:{}", server.port()),
            Some(vec![KeyValue {
                key: "authorization".to_string(),
                value: "Bearer 123".to_string(),
            }]),
            runtime,
        );

        let resp = grpc_reflection.list_all_files().await?;

        assert_eq!(
            [
                "news.NewsService".to_string(),
                "grpc.reflection.v1alpha.ServerReflection".to_string()
            ]
            .to_vec(),
            resp
        );

        http_reflection_service_not_found.assert();

        Ok(())
    }
}
