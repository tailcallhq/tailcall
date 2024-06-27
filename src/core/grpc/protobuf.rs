use std::fmt::Debug;

use anyhow::{anyhow, bail, Context, Result};
use async_graphql::Value;
use prost::bytes::BufMut;
use prost::Message;
use prost_reflect::prost_types::FileDescriptorSet;
use prost_reflect::{
    DescriptorPool, DynamicMessage, MessageDescriptor, MethodDescriptor, SerializeOptions,
    ServiceDescriptor,
};
use serde_json::Deserializer;

use crate::core::blueprint::GrpcMethod;

fn to_message(descriptor: &MessageDescriptor, input: &str) -> Result<DynamicMessage> {
    let mut deserializer = Deserializer::from_str(input);
    let message =
        DynamicMessage::deserialize(descriptor.clone(), &mut deserializer).with_context(|| {
            format!(
                "Failed to parse input according to type {}",
                descriptor.full_name()
            )
        })?;
    deserializer.end()?;

    Ok(message)
}

fn message_to_bytes(message: DynamicMessage) -> Result<Vec<u8>> {
    let mut buf: Vec<u8> = Vec::with_capacity(message.encoded_len() + 5);
    // set compression flag
    buf.put_u8(0);
    // next 4 bytes should encode message length
    buf.put_u32(message.encoded_len() as u32);
    // encode the message itself
    message.encode(&mut buf)?;

    Ok(buf)
}

pub fn protobuf_value_as_str(value: &prost_reflect::Value) -> String {
    use prost_reflect::Value;

    match value {
        Value::I32(v) => v.to_string(),
        Value::I64(v) => v.to_string(),
        Value::U32(v) => v.to_string(),
        Value::U64(v) => v.to_string(),
        Value::F32(v) => v.to_string(),
        Value::F64(v) => v.to_string(),
        Value::EnumNumber(v) => v.to_string(),
        Value::String(s) => s.clone(),
        _ => Default::default(),
    }
}

pub fn get_field_value_as_str(message: &DynamicMessage, field_name: &str) -> Result<String> {
    let field = message
        .get_field_by_name(field_name)
        .ok_or(anyhow!("Unable to find key"))?;

    Ok(protobuf_value_as_str(&field))
}

#[derive(Debug)]
pub struct ProtobufSet {
    descriptor_pool: DescriptorPool,
}

// TODO: support for reflection
impl ProtobufSet {
    // TODO: load definitions from proto file for now, but in future
    // it could be more convenient to load FileDescriptorSet instead
    // either from file or server reflection
    pub fn from_proto_file(file_descriptor_set: FileDescriptorSet) -> Result<Self> {
        let descriptor_pool = DescriptorPool::from_file_descriptor_set(file_descriptor_set)?;
        Ok(Self { descriptor_pool })
    }

    pub fn find_service(&self, grpc_method: &GrpcMethod) -> Result<ProtobufService> {
        let service_name = format!("{}.{}", grpc_method.package, grpc_method.service);

        let service_descriptor = self
            .descriptor_pool
            .get_service_by_name(&service_name)
            .with_context(|| format!("Couldn't find definitions for service {service_name}"))?;

        Ok(ProtobufService { service_descriptor })
    }
}

#[derive(Debug, Clone)]
pub struct ProtobufMessage {
    pub message_descriptor: MessageDescriptor,
}

impl ProtobufMessage {
    pub fn decode(&self, bytes: &[u8]) -> Result<Value> {
        let message = DynamicMessage::decode(self.message_descriptor.clone(), bytes)?;

        let json = serde_json::to_value(message)?;

        Ok(async_graphql::Value::from_json(json)?)
    }
}

#[derive(Debug)]
pub struct ProtobufService {
    service_descriptor: ServiceDescriptor,
}

impl ProtobufService {
    pub fn find_operation(&self, grpc_method: &GrpcMethod) -> Result<ProtobufOperation> {
        let method = self
            .service_descriptor
            .methods()
            .find(|method| method.name() == grpc_method.name)
            .with_context(|| format!("Couldn't find method {}", grpc_method.name))?;

        let input_type = method.input();
        let output_type = method.output();

        Ok(ProtobufOperation::new(method, input_type, output_type))
    }
}

#[derive(Debug, Clone)]
pub struct ProtobufOperation {
    pub method: MethodDescriptor,
    pub input_type: MessageDescriptor,
    pub output_type: MessageDescriptor,
    serialize_options: SerializeOptions,
}

impl Eq for ProtobufOperation {}

impl PartialEq for ProtobufOperation {
    fn eq(&self, other: &Self) -> bool {
        self.method.eq(&other.method)
            && self.input_type.eq(&other.input_type)
            && self.output_type.eq(&other.output_type)
    }
}

// TODO: support compression
impl ProtobufOperation {
    pub fn new(
        method: MethodDescriptor,
        input_type: MessageDescriptor,
        output_type: MessageDescriptor,
    ) -> Self {
        Self {
            method,
            input_type,
            output_type,
            serialize_options: SerializeOptions::default().skip_default_fields(false),
        }
    }
    pub fn name(&self) -> &str {
        self.method.name()
    }

    pub fn service_name(&self) -> &str {
        self.method.parent_service().name()
    }

    pub fn convert_input(&self, input: &str) -> Result<Vec<u8>> {
        let message = to_message(&self.input_type, input)?;

        message_to_bytes(message)
    }

    pub fn convert_multiple_inputs<'a>(
        &self,
        child_inputs: impl Iterator<Item = &'a str>,
        id: &str,
    ) -> Result<(Vec<u8>, Vec<String>)> {
        // Find the field of list type that should hold child messages
        let field_descriptor = self
            .input_type
            .fields()
            .find(|field| field.is_list())
            .ok_or(anyhow!("Unable to find list field on type"))?;
        let field_kind = field_descriptor.kind();
        let child_message_descriptor = field_kind
            .as_message()
            .ok_or(anyhow!("Couldn't resolve message"))?;
        let mut message = DynamicMessage::new(self.input_type.clone());

        let child_messages = child_inputs
            .map(|input| to_message(child_message_descriptor, input))
            .collect::<Result<Vec<DynamicMessage>>>()?;

        let ids = child_messages
            .iter()
            .map(|message| get_field_value_as_str(message, id))
            .collect::<Result<Vec<String>>>()?;

        message.set_field(
            &field_descriptor,
            prost_reflect::Value::List(
                child_messages
                    .into_iter()
                    .map(prost_reflect::Value::Message)
                    .collect(),
            ),
        );

        message_to_bytes(message).map(|result| (result, ids))
    }

    pub fn convert_output<T: serde::de::DeserializeOwned>(&self, bytes: &[u8]) -> Result<T> {
        if bytes.len() < 5 {
            bail!("Empty response");
        }
        // ignore 5 first bytes as they are part of Length-Prefixed Message Framing
        // see https://www.oreilly.com/library/view/grpc-up-and/9781492058328/ch04.html#:~:text=Length%2DPrefixed%20Message%20Framing
        // 1st byte - compression flag
        // 2-4th bytes - length of the message
        let message =
            DynamicMessage::decode(self.output_type.clone(), &bytes[5..]).with_context(|| {
                format!(
                    "Failed to parse response for type {}",
                    self.output_type.full_name()
                )
            })?;

        let mut serializer = serde_json::Serializer::new(vec![]);
        message.serialize_with_options(&mut serializer, &self.serialize_options)?;
        let json = serde_json::from_slice::<T>(serializer.into_inner().as_ref())?;
        Ok(json)
    }

    pub fn find_message(&self, name: &str) -> Option<ProtobufMessage> {
        let message_descriptor = self.method.parent_pool().get_message_by_name(name)?;

        Some(ProtobufMessage { message_descriptor })
    }
}

#[cfg(test)]
pub mod tests {
    use std::path::Path;

    use anyhow::Result;
    use prost_reflect::Value;
    use serde_json::json;
    use tailcall_fixtures::protobuf;

    use super::*;
    use crate::core::blueprint::GrpcMethod;
    use crate::core::config::reader::ConfigReader;
    use crate::core::config::{Config, Field, Grpc, Link, LinkType, Type};

    pub async fn get_proto_file(path: &str) -> Result<FileDescriptorSet> {
        let runtime = crate::core::runtime::test::init(None);
        let reader = ConfigReader::init(runtime);

        let id = Path::new(path)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        let mut config = Config::default().links(vec![Link {
            id: Some(id.clone()),
            src: path.to_string(),
            type_of: LinkType::Protobuf,
        }]);

        let method = GrpcMethod { package: id, service: "a".to_owned(), name: "b".to_owned() };
        let grpc = Grpc { method: method.to_string(), ..Default::default() };
        config.types.insert(
            "foo".to_string(),
            Type::default().fields(vec![("bar", Field::default().grpc(grpc))]),
        );
        Ok(reader
            .resolve(config, None)
            .await?
            .extensions()
            .get_file_descriptor_set())
    }

    #[test]
    fn convert_value() {
        assert_eq!(
            protobuf_value_as_str(&Value::String("test string".to_owned())),
            "test string".to_owned()
        );
        assert_eq!(protobuf_value_as_str(&Value::I32(25)), "25".to_owned());
        assert_eq!(protobuf_value_as_str(&Value::F32(1.25)), "1.25".to_owned());
        assert_eq!(protobuf_value_as_str(&Value::I64(35)), "35".to_owned());
        assert_eq!(protobuf_value_as_str(&Value::F64(3.38)), "3.38".to_owned());
        assert_eq!(
            protobuf_value_as_str(&Value::EnumNumber(55)),
            "55".to_owned()
        );
        assert_eq!(protobuf_value_as_str(&Value::Bool(false)), "".to_owned());
        assert_eq!(
            protobuf_value_as_str(&Value::Map(Default::default())),
            "".to_owned()
        );
        assert_eq!(
            protobuf_value_as_str(&Value::List(Default::default())),
            "".to_owned()
        );
        assert_eq!(
            protobuf_value_as_str(&Value::Bytes(Default::default())),
            "".to_owned()
        );
    }

    #[tokio::test]
    async fn unknown_file() -> Result<()> {
        let error = get_proto_file("_unknown.proto").await;
        assert!(error.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn service_not_found() -> Result<()> {
        let grpc_method = GrpcMethod::try_from("greetings._unknown.foo").unwrap();
        let file = ProtobufSet::from_proto_file(get_proto_file(protobuf::GREETINGS).await?)?;
        let error = file.find_service(&grpc_method).unwrap_err();

        assert_eq!(
            error.to_string(),
            "Couldn't find definitions for service greetings._unknown"
        );

        Ok(())
    }

    #[tokio::test]
    async fn method_not_found() -> Result<()> {
        let grpc_method = GrpcMethod::try_from("greetings.Greeter._unknown").unwrap();
        let file = ProtobufSet::from_proto_file(get_proto_file(protobuf::GREETINGS).await?)?;
        let service = file.find_service(&grpc_method)?;
        let error = service.find_operation(&grpc_method).unwrap_err();

        assert_eq!(error.to_string(), "Couldn't find method _unknown");

        Ok(())
    }

    #[tokio::test]
    async fn greetings_proto_file() -> Result<()> {
        let grpc_method = GrpcMethod::try_from("greetings.Greeter.SayHello").unwrap();
        let file = ProtobufSet::from_proto_file(get_proto_file(protobuf::GREETINGS).await?)?;
        let service = file.find_service(&grpc_method)?;
        let operation = service.find_operation(&grpc_method)?;

        let output = b"\0\0\0\0\x0e\n\x0ctest message";

        let parsed = operation.convert_output::<serde_json::Value>(output)?;

        assert_eq!(
            serde_json::to_value(parsed)?,
            json!({
              "message": "test message"
            })
        );

        Ok(())
    }

    #[tokio::test]
    async fn news_proto_file() -> Result<()> {
        let grpc_method = GrpcMethod::try_from("news.NewsService.GetNews").unwrap();

        let file = ProtobufSet::from_proto_file(get_proto_file(protobuf::NEWS).await?)?;
        let service = file.find_service(&grpc_method)?;
        let operation = service.find_operation(&grpc_method)?;

        let input = operation.convert_input(r#"{ "id": 1 }"#)?;

        assert_eq!(input, b"\0\0\0\0\x02\x08\x01");

        let output = b"\0\0\0\x005\x08\x01\x12\x06Note 1\x1a\tContent 1\"\x0cPost image 1";

        let parsed = operation.convert_output::<serde_json::Value>(output)?;

        assert_eq!(
            serde_json::to_value(parsed)?,
            json!({
              "id": 1, "title": "Note 1", "body": "Content 1", "postImage": "Post image 1", "status": "PUBLISHED"
            })
        );

        Ok(())
    }

    #[tokio::test]
    async fn oneof_proto_file() -> Result<()> {
        let grpc_method = GrpcMethod::try_from("oneof.OneOfService.GetOneOf").unwrap();

        let file = ProtobufSet::from_proto_file(get_proto_file(protobuf::ONEOF).await?)?;
        let service = file.find_service(&grpc_method)?;
        let operation = service.find_operation(&grpc_method)?;

        let input = operation.convert_input(r#"{ "payload": { "payload": "test" } }"#)?;

        assert_eq!(input, b"\0\0\0\0\x08\x12\x06\n\x04test");

        let input =
            operation.convert_input(r#"{ "usual": "str", "command": { "command": "call" } }"#)?;

        assert_eq!(input, b"\0\0\0\0\r\n\x03str\x1a\x06\n\x04call");

        let output = b"\0\0\0\0\x08\x12\x06\n\x04body";

        let parsed = operation.convert_output::<serde_json::Value>(output)?;

        assert_eq!(
            serde_json::to_value(parsed)?,
            json!({
                "usual": 0,
                "payload": { "payload": "body" }
            })
        );

        let output = b"\0\0\0\0\x09\x08\x05\x1A\x05\n\x03end";

        let parsed = operation.convert_output::<serde_json::Value>(output)?;

        assert_eq!(
            serde_json::to_value(parsed)?,
            json!({
                "usual": 5,
                "command": { "command": "end" }
            })
        );
        let output = b"\0\0\0\0\x09\x22\x07content";

        let parsed = operation.convert_output::<serde_json::Value>(output)?;

        assert_eq!(
            serde_json::to_value(parsed)?,
            json!({
                "usual": 0,
                "response": "content"
            })
        );

        Ok(())
    }

    #[tokio::test]
    async fn news_proto_file_multiple_messages() -> Result<()> {
        let grpc_method = GrpcMethod::try_from("news.NewsService.GetMultipleNews").unwrap();
        let file = ProtobufSet::from_proto_file(get_proto_file(protobuf::NEWS).await?)?;
        let service = file.find_service(&grpc_method)?;
        let multiple_operation = service.find_operation(&grpc_method)?;

        let child_messages = vec![r#"{ "id": 3 }"#, r#"{ "id": 5 }"#, r#"{ "id": 1 }"#];

        let (multiple_message, grouped) =
            multiple_operation.convert_multiple_inputs(child_messages.into_iter(), "id")?;

        assert_eq!(
            multiple_message,
            b"\0\0\0\0\x0c\n\x02\x08\x03\n\x02\x08\x05\n\x02\x08\x01"
        );
        assert_eq!(
            grouped,
            vec!["3".to_owned(), "5".to_owned(), "1".to_owned()]
        );

        let output = b"\0\0\0\0s\n#\x08\x01\x12\x06Note 1\x1a\tContent 1\"\x0cPost image 1\n%\x08\x03\x12\x06Note 3\x1a\tContent 3\"\x0cPost image 3(\x01\n%\x08\x05\x12\x06Note 5\x1a\tContent 5\"\x0cPost image 5(\x02";

        let parsed = multiple_operation.convert_output::<serde_json::Value>(output)?;

        assert_eq!(
            serde_json::to_value(parsed)?,
            json!({
                "news": [
                    { "id": 1, "title": "Note 1", "body": "Content 1", "postImage": "Post image 1", "status": "PUBLISHED" },
                    { "id": 3, "title": "Note 3", "body": "Content 3", "postImage": "Post image 3", "status": "DRAFT" },
                    { "id": 5, "title": "Note 5", "body": "Content 5", "postImage": "Post image 5", "status": "DELETED" },
                  ]
            })
        );

        Ok(())
    }

    #[tokio::test]
    async fn map_proto_file() -> Result<()> {
        let grpc_method = GrpcMethod::try_from("map.MapService.GetMap").unwrap();

        let file = ProtobufSet::from_proto_file(get_proto_file(protobuf::MAP).await?)?;
        let service = file.find_service(&grpc_method)?;
        let operation = service.find_operation(&grpc_method)?;

        // only single key-value in json since the converted output can change the
        // ordering on every run
        let input = operation.convert_input(r#"{ "map": { "key": "value" } }"#)?;

        assert_eq!(input, b"\0\0\0\0\x0e\n\x0c\n\x03key\x12\x05value");

        let output = b"\0\0\0\0\x12\n\t\x08\x01\x12\x05value\n\x05\x08\x02\x12\x01v";

        let parsed = operation.convert_output::<serde_json::Value>(output)?;

        assert_eq!(
            serde_json::to_value(parsed)?,
            json!({
              "map": { "1": "value", "2": "v" }
            })
        );

        Ok(())
    }

    #[tokio::test]
    async fn optional_proto_file() -> Result<()> {
        let grpc_method = GrpcMethod::try_from("type.TypeService.Get").unwrap();

        let file = ProtobufSet::from_proto_file(get_proto_file(protobuf::OPTIONAL).await?)?;
        let service = file.find_service(&grpc_method)?;
        let operation = service.find_operation(&grpc_method)?;

        let input = operation.convert_input(r#"{ }"#)?;

        assert_eq!(input, b"\0\0\0\0\0");

        let output = b"\0\0\0\0\0";

        let parsed = operation.convert_output::<serde_json::Value>(output)?;

        assert_eq!(
            serde_json::to_value(parsed)?,
            json!({"id": 0, "str": "", "num": [], "nestedRep": []})
        );

        let output = b"\0\0\0\0\x03\x92\x03\0";

        let parsed = operation.convert_output::<serde_json::Value>(output)?;

        assert_eq!(
            serde_json::to_value(parsed)?,
            json!({"id": 0, "str": "", "num": [], "nestedRep": [], "nested": {"id": 0, "str": "", "num": []}})
        );

        Ok(())
    }

    #[tokio::test]
    async fn scalars_proto_file() -> Result<()> {
        let grpc_method = GrpcMethod::try_from("scalars.Example.Get").unwrap();

        let file = ProtobufSet::from_proto_file(get_proto_file(protobuf::SCALARS).await?)?;
        let service = file.find_service(&grpc_method)?;
        let operation = service.find_operation(&grpc_method)?;

        // numbers as numbers in json
        let input = operation
            .convert_input(r#"{ "boolean": true, "doubleNum": 3.25, "fixedint64": 1248645648 }"#)?;

        assert_eq!(
            input,
            b"\0\0\0\0\x14\x08\x01\x19\0\0\0\0\0\0\n@)\x10\xd2lJ\0\0\0\0"
        );

        // the same output as input result from above to verify conversion
        let output = b"\0\0\0\0\x16\n\x14\x08\x01\x19\0\0\0\0\0\0\n@)\x10\xd2lJ\0\0\0\0";

        let parsed = operation.convert_output::<serde_json::Value>(output)?;

        assert_eq!(
            serde_json::to_value(parsed)?,
            json!({
              "result": [{
                "boolean": true,
                "bytesType": "",
                "doubleNum": 3.25,
                "fixedint32": 0,
                // by default, prost outputs 64bit integers as strings
                "fixedint64": "1248645648",
                "floatNum": 0.0,
                "integer32": 0,
                "integer64": "0",
                "sfixedint32": 0,
                "sfixedint64": "0",
                "sinteger32": 0,
                "sinteger64": "0",
                "str": "",
                "uinteger32": 0,
                "uinteger64": "0"
              }]
            })
        );

        // numbers as string in json
        let input = operation.convert_input(
            r#"{ "integer32": "125", "sinteger64": "-18564864651", "uinteger64": "464646694646" }"#,
        )?;

        assert_eq!(
            input,
            b"\0\0\0\0\x108}`\x95\xea\xea\xa8\x8a\x01x\xf6\xcd\xe7\xf8\xc2\r"
        );

        // the same output as input result from above to verify conversion
        let output = b"\0\0\0\0\x12\n\x108}`\x95\xea\xea\xa8\x8a\x01x\xf6\xcd\xe7\xf8\xc2\r";

        let parsed = operation.convert_output::<serde_json::Value>(output)?;

        assert_eq!(
            serde_json::to_value(parsed)?,
            json!({
              "result": [{
                "boolean": false,
                "bytesType": "",
                "doubleNum": 0.0,
                "fixedint32": 0,
                "fixedint64": "0",
                "floatNum": 0.0,
                "integer32": 125,
                "integer64": "0",
                "sfixedint32": 0,
                "sfixedint64": "0",
                "sinteger32": 0,
                "sinteger64": "-18564864651",
                "str": "",
                "uinteger32": 0,
                "uinteger64": "464646694646"
              }]
            })
        );

        // numbers out of range
        let input: anyhow::Error = operation
            .convert_input(
                r#"{
                "floatNum": 1e154561.14848449464654948484542189,
                "integer32": 32147483647,
                "sinteger32": "32147483647",
                "integer64": "4894654899848486451568418645165486465"
            }"#,
            )
            .unwrap_err();

        assert_eq!(
            input.to_string(),
            "Failed to parse input according to type scalars.Item"
        );

        Ok(())
    }
}
