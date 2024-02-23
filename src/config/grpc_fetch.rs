use anyhow::{Context, Result};
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use hyper::Method;
use nom::AsBytes;
use prost::Message;
use prost_reflect::prost_types::{FileDescriptorProto, FileDescriptorSet};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::grpc::protobuf::ProtobufSet;
use crate::runtime::TargetRuntime;

const REFLECTION_PROTO: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/proto/reflection.proto"
));

/// This function is just used for better exception handling
fn get_protobuf_set() -> Result<ProtobufSet> {
    let descriptor = protox_parse::parse("reflection", REFLECTION_PROTO)?;
    let mut descriptor_set = FileDescriptorSet::default();
    descriptor_set.file.push(descriptor);
    ProtobufSet::from_proto_file(&descriptor_set)
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct OriginalRequest {
    list_services: String,
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
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct CustomResponse {
    original_request: OriginalRequest,
    list_services_response: ListServicesResponse,
}

/// Makes `ListService` request to the grpc reflection server
pub async fn list_all_files(url: &str, target_runtime: &TargetRuntime) -> Result<Vec<String>> {
    let protobuf_set = get_protobuf_set()?;

    // let mut methods = vec![];
    let mut url: url::Url = url.parse()?;
    url.set_path("grpc.reflection.v1alpha.ServerReflection/ServerReflectionInfo");

    let mut req = reqwest::Request::new(Method::POST, url);
    *req.body_mut() = Some(reqwest::Body::from(b"\0\0\0\0\x02:\0".to_vec())); // magic :)

    let resp = target_runtime.http.execute(req).await?;
    let body = resp.body.as_bytes();

    let reflection_service =
        protobuf_set.find_service("grpc.reflection.v1alpha.ServerReflection")?;
    let operation = reflection_service.find_operation("ServerReflectionInfo")?;

    let response: Value = serde_json::from_value(operation.convert_output(body)?.into_json()?)?;
    println!("{}",response);
    let response: CustomResponse = serde_json::from_value(response)?;

    // Extracting names from services
    let methods: Vec<String> = response
        .list_services_response
        .service
        .iter()
        .map(|s| s.name.clone())
        .collect();

    Ok(methods)
}

/// Makes `Get Service` request to the grpc reflection server

pub async fn get_by_service(
    url: &str,
    target_runtime: &TargetRuntime,
    service: &str,
) -> Result<FileDescriptorProto> {
    let protobuf_set = get_protobuf_set()?;

    let mut url: url::Url = url.parse()?;
    url.set_path("grpc.reflection.v1alpha.ServerReflection/ServerReflectionInfo");

    let mut req = reqwest::Request::new(Method::POST, url);
    *req.body_mut() = Some(reqwest::Body::from(format!("\0\0\0\0\x12\"\x10{service}")));

    let resp = target_runtime.http.execute(req).await?;
    let body = resp.body.as_bytes();

    let reflection_service =
        protobuf_set.find_service("grpc.reflection.v1alpha.ServerReflection")?;
    let operation = reflection_service.find_operation("ServerReflectionInfo")?;

    let response: Value = serde_json::from_value(operation.convert_output(body)?.into_json()?)?;
    let object = response
        .as_object()
        .context("Invalid response, expected object.")?;
    let file_descriptor_response = object
        .get("fileDescriptorResponse")
        .context("Expected key fileDescriptorResponse but found none")?;
    let file_descriptor_response_object = file_descriptor_response
        .as_object()
        .context("Expected fileDescriptorResponse to be an object")?;
    let file_descriptor_proto = file_descriptor_response_object
        .get("fileDescriptorProto")
        .context("expected fileDescriptorProto as object")?;
    let file_descriptor_proto_arr = file_descriptor_proto
        .as_array()
        .context("Expected fileDescriptorProto to be an array")?;
    let file_descriptor_proto = file_descriptor_proto_arr
        .first()
        .context("Received empty fileDescriptorProto")?;

    let file_descriptor_proto = file_descriptor_proto
        .as_str()
        .context("Expected content of fileDescriptorProto as a string")?;
    let file_descriptor_proto =
        FileDescriptorProto::decode(BASE64_STANDARD.decode(file_descriptor_proto)?.as_bytes())?;

    Ok(file_descriptor_proto)
}

/// Makes `Get Proto/Symbol Name` request to the grpc reflection server
pub async fn get_by_proto_name(
    url: &str,
    target_runtime: &TargetRuntime,
    proto_name: &str,
) -> Result<FileDescriptorProto> {
    let protobuf_set = get_protobuf_set()?;

    let mut url: url::Url = url.parse()?;
    url.set_path("grpc.reflection.v1alpha.ServerReflection/ServerReflectionInfo");

    let mut req = reqwest::Request::new(Method::POST, url);
    *req.body_mut() = Some(reqwest::Body::from(format!(
        "\0\0\0\0\x0c\x1a\n{}",
        proto_name
    )));

    let resp = target_runtime.http.execute(req).await?;
    let body = resp.body.as_bytes();

    let reflection_service =
        protobuf_set.find_service("grpc.reflection.v1alpha.ServerReflection")?;
    let operation = reflection_service.find_operation("ServerReflectionInfo")?;

    let response: Value = serde_json::from_value(operation.convert_output(body)?.into_json()?)?;
    let object = response
        .as_object()
        .context("Invalid response, expected object.")?;
    let file_descriptor_response = object
        .get("fileDescriptorResponse")
        .context("Expected key fileDescriptorResponse but found none")?;
    let file_descriptor_response_object = file_descriptor_response
        .as_object()
        .context("Expected fileDescriptorResponse to be an object")?;
    let file_descriptor_proto = file_descriptor_response_object
        .get("fileDescriptorProto")
        .context("expected fileDescriptorProto as object")?;
    let file_descriptor_proto_arr = file_descriptor_proto
        .as_array()
        .context("Expected fileDescriptorProto to be an array")?;
    let file_descriptor_proto = file_descriptor_proto_arr
        .first()
        .context("Received empty fileDescriptorProto")?;

    let file_descriptor_proto = file_descriptor_proto
        .as_str()
        .context("Expected content of fileDescriptorProto as a string")?;
    let file_descriptor_proto =
        FileDescriptorProto::decode(BASE64_STANDARD.decode(file_descriptor_proto)?.as_bytes())?;

    Ok(file_descriptor_proto)
}
