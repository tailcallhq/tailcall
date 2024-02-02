use std::fmt::Debug;

use anyhow::{anyhow, bail, Context, Result};
use async_graphql::Value;
use prost::bytes::BufMut;
use prost::Message;
use prost_reflect::prost_types::FileDescriptorSet;
use prost_reflect::{
    DescriptorPool, DynamicMessage, MessageDescriptor, MethodDescriptor, ServiceDescriptor,
};
use serde_json::Deserializer;

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
    pub fn from_proto_file(file_descriptor_set: &FileDescriptorSet) -> Result<Self> {
        let descriptor_pool =
            DescriptorPool::from_file_descriptor_set(file_descriptor_set.clone())?;
        Ok(Self { descriptor_pool })
    }

    pub fn find_service(&self, name: &str) -> Result<ProtobufService> {
        let service_descriptor = self
            .descriptor_pool
            .get_service_by_name(name)
            .with_context(|| format!("Couldn't find definitions for service {name}"))?;

        Ok(ProtobufService { service_descriptor })
    }
}

#[derive(Debug)]
pub struct ProtobufService {
    service_descriptor: ServiceDescriptor,
}

impl ProtobufService {
    pub fn find_operation(&self, method_name: &str) -> Result<ProtobufOperation> {
        let method = self
            .service_descriptor
            .methods()
            .find(|method| method.name() == method_name)
            .with_context(|| format!("Couldn't find method {method_name}"))?;

        let input_type = method.input();
        let output_type = method.output();

        Ok(ProtobufOperation { method, input_type, output_type })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtobufOperation {
    method: MethodDescriptor,
    pub input_type: MessageDescriptor,
    pub output_type: MessageDescriptor,
}

// TODO: support compression
impl ProtobufOperation {
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

    pub fn convert_output(&self, bytes: &[u8]) -> Result<Value> {
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

        let json = serde_json::to_value(message)?;

        Ok(async_graphql::Value::from_json(json)?)
    }
}

#[cfg(test)]
mod tests {
    // TODO: Rewrite protobuf tests
    use std::path::PathBuf;

    use anyhow::Result;
    use once_cell::sync::Lazy;
    use prost_reflect::Value;
    use serde_json::json;

    use super::*;
    use crate::config::reader::ConfigReader;
    use crate::config::{Config, Field, Grpc, Type, Upstream};

    use std::collections::HashMap;
    use std::sync::Arc;
    use anyhow::anyhow;
    use async_trait::async_trait;
    use hyper::body::Bytes;
    use reqwest::{Client, Request};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use crate::{EnvIO, HttpIO};
    use crate::cache::InMemoryCache;
    use crate::http::Response;
    use crate::target_runtime::TargetRuntime;

    pub struct Env {
        env: HashMap<String, String>,
    }

    #[derive(Clone)]
    pub struct FileIO {}

    impl FileIO {
        pub fn init() -> Self {
            FileIO {}
        }
    }

    #[async_trait::async_trait]
    impl crate::FileIO for FileIO {
        async fn write<'a>(&'a self, path: &'a str, content: &'a [u8]) -> anyhow::Result<()> {
            let mut file = tokio::fs::File::create(path).await?;
            file.write_all(content).await.map_err(|e|anyhow!("{}",e))?;
            log::info!("File write: {} ... ok", path);
            Ok(())
        }

        async fn read<'a>(&'a self, path: &'a str) -> anyhow::Result<String> {
            let mut file = tokio::fs::File::open(path).await?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)
                .await
                .map_err(|e|anyhow!("{}",e))?;
            log::info!("File read: {} ... ok", path);
            Ok(String::from_utf8(buffer)?)
        }
    }


    impl EnvIO for Env {
        fn get(&self, key: &str) -> Option<String> {
            self.env.get(key).cloned()
        }
    }

    impl Env {
        pub fn init(map: HashMap<String, String>) -> Self {
            Self { env: map }
        }
    }

    struct Http {
        client: Client
    }
    #[async_trait]
    impl HttpIO for Http {
        async fn execute(&self, request: Request) -> anyhow::Result<Response<Bytes>> {
            let resp = self.client.execute(request).await?;
            let resp = crate::http::Response::from_reqwest(resp).await?;
            Ok(resp)
        }
    }

    fn init_runtime() -> TargetRuntime {
        let http = Arc::new(Http{ client: Client::new() });
        let http2_only = http.clone();
        TargetRuntime {
            http,
            http2_only,
            env: Arc::new(Env::init(HashMap::new())),
            file: Arc::new(FileIO::init()),
            cache: Arc::new(InMemoryCache::new()),
        }
    }

    static TEST_DIR: Lazy<PathBuf> = Lazy::new(|| {
        let root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let mut test_dir = root_dir.join(file!());

        test_dir.pop();
        test_dir.push("tests");

        test_dir
    });

    fn get_test_file(name: &str) -> PathBuf {
        let mut test_file = TEST_DIR.clone();

        test_file.push(name);
        test_file
    }

    async fn get_proto_file(name: &str) -> Result<FileDescriptorSet> {
        let runtime = init_runtime();
        let reader = ConfigReader::init(runtime);
        let mut config = Config::default();
        let grpc = Grpc {
            proto_path: get_test_file(name)
                .to_str()
                .context("Failed to parse or load proto file")?
                .to_string(),
            ..Default::default()
        };
        config.types.insert(
            "foo".to_string(),
            Type::default().fields(vec![("bar", Field::default().grpc(grpc))]),
        );
        Ok(reader
            .resolve(config)
            .await?
            .extensions
            .grpc_file_descriptor)
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
        let error = get_proto_file("_unknown.proto").await.unwrap_err();

        assert_eq!(
            error.to_string(),
            "No such file or directory (os error 2)".to_string()
        );

        Ok(())
    }

    #[tokio::test]
    async fn service_not_found() -> Result<()> {
        let file = ProtobufSet::from_proto_file(&get_proto_file("greetings.proto").await?)?;
        let error = file.find_service("_unknown").unwrap_err();

        assert_eq!(
            error.to_string(),
            "Couldn't find definitions for service _unknown"
        );

        Ok(())
    }

    #[tokio::test]
    async fn method_not_found() -> Result<()> {
        let file = ProtobufSet::from_proto_file(&get_proto_file("greetings.proto").await?)?;
        let service = file.find_service("Greeter")?;
        let error = service.find_operation("_unknown").unwrap_err();

        assert_eq!(error.to_string(), "Couldn't find method _unknown");

        Ok(())
    }

    #[tokio::test]
    async fn greetings_proto_file() -> Result<()> {
        let file = ProtobufSet::from_proto_file(&get_proto_file("greetings.proto").await?)?;
        let service = file.find_service("Greeter")?;
        let operation = service.find_operation("SayHello")?;

        let output = b"\0\0\0\0\x0e\n\x0ctest message";

        let parsed = operation.convert_output(output)?;

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
        let file = ProtobufSet::from_proto_file(&get_proto_file("news.proto").await?)?;
        let service = file.find_service("NewsService")?;
        let operation = service.find_operation("GetNews")?;

        let input = operation.convert_input(r#"{ "id": 1 }"#)?;

        assert_eq!(input, b"\0\0\0\0\x02\x08\x01");

        let output = b"\0\0\0\x005\x08\x01\x12\x06Note 1\x1a\tContent 1\"\x0cPost image 1";

        let parsed = operation.convert_output(output)?;

        assert_eq!(
            serde_json::to_value(parsed)?,
            json!({
              "id": 1, "title": "Note 1", "body": "Content 1", "postImage": "Post image 1"
            })
        );

        Ok(())
    }

    #[tokio::test]
    async fn news_proto_file_multiple_messages() -> Result<()> {
        let file = ProtobufSet::from_proto_file(&get_proto_file("news.proto").await?)?;
        let service = file.find_service("NewsService")?;
        let multiple_operation = service.find_operation("GetMultipleNews")?;

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

        let output = b"\0\0\0\0o\n#\x08\x01\x12\x06Note 1\x1a\tContent 1\"\x0cPost image 1\n#\x08\x03\x12\x06Note 3\x1a\tContent 3\"\x0cPost image 3\n#\x08\x05\x12\x06Note 5\x1a\tContent 5\"\x0cPost image 5";

        let parsed = multiple_operation.convert_output(output)?;

        assert_eq!(
            serde_json::to_value(parsed)?,
            json!({
              "news": [
                { "id": 1, "title": "Note 1", "body": "Content 1", "postImage": "Post image 1" },
                { "id": 3, "title": "Note 3", "body": "Content 3", "postImage": "Post image 3" },
                { "id": 5, "title": "Note 5", "body": "Content 5", "postImage": "Post image 5" },
              ]
            })
        );

        Ok(())
    }
}
