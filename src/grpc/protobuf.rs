// mod conversion;

use std::fmt::Debug;
use std::path::Path;

use anyhow::{bail, Context, Result};
use async_graphql::Value;
use prost::{bytes::BufMut, Message};
use prost_reflect::{DescriptorPool, DynamicMessage, MessageDescriptor, ServiceDescriptor};
use serde_json::Deserializer;

#[derive(Debug)]
pub struct ProtobufSet {
  descriptor_pool: DescriptorPool,
}

impl ProtobufSet {
  // TODO: load definitions from proto file for now, but in future
  // it could be more convenient to load FileDescriptorSet instead
  // either from file or server reflection
  pub fn from_proto_file(proto_path: &Path) -> Result<Self> {
    let parent_dir = proto_path
      .parent()
      .context("Failed to resolve parent dir for proto file")?;

    let file_descriptor_set = protox::compile(&[proto_path], &[parent_dir])
      .with_context(|| format!("Failed to parse proto file {}", proto_path.display()))?;

    let descriptor_pool = DescriptorPool::from_file_descriptor_set(file_descriptor_set)?;

    Ok(Self { descriptor_pool })
  }
}

#[derive(Debug)]
pub struct ProtobufService {
  service_descriptor: ServiceDescriptor,
}

// TODO: support for reflection
impl ProtobufService {
  pub fn new(file: &ProtobufSet, name: &str) -> Result<ProtobufService> {
    let service_descriptor = file
      .descriptor_pool
      .get_service_by_name(name)
      .with_context(|| format!("Couldn't find definitions for service {name}"))?;

    Ok(Self { service_descriptor })
  }
}

#[derive(Debug, Clone)]
pub struct ProtobufOperation {
  input_type: MessageDescriptor,
  output_type: MessageDescriptor,
}

// TODO: support compression
impl ProtobufOperation {
  pub fn new(service: &ProtobufService, method_name: &str) -> Result<Self> {
    let method = service
      .service_descriptor
      .methods()
      .find(|method| method.name() == method_name)
      .with_context(|| format!("Could't find method {method_name}"))?;

    let input_type = method.input();
    let output_type = method.output();

    Ok(Self { input_type, output_type })
  }

  pub fn convert_input(&self, input_json: &str) -> Result<Vec<u8>> {
    let mut deserializer = Deserializer::from_str(input_json);
    let message = DynamicMessage::deserialize(self.input_type.clone(), &mut deserializer)?;
    deserializer.end()?;
    let mut buf: Vec<u8> = Vec::with_capacity(message.encoded_len() + 5);
    // set compression flag
    buf.put_u8(0);
    // next 4 bytes should encode message length
    buf.put_u32(message.encoded_len() as u32);
    // encode the message itself
    message.encode(&mut buf)?;

    Ok(buf)
  }

  pub fn convert_output(&self, bytes: &[u8]) -> Result<Value> {
    if bytes.len() < 5 {
      bail!("Empty response");
    }
    // ignore 5 first bytes as they are part of Length-Prefixed Message Framing
    // see https://www.oreilly.com/library/view/grpc-up-and/9781492058328/ch04.html#:~:text=Length%2DPrefixed%20Message%20Framing
    // 1st byte - compression flag
    // 2-4th bytes - length of the message
    let message = DynamicMessage::decode(self.output_type.clone(), &bytes[5..])?;

    let json = serde_json::to_value(message)?;

    Ok(async_graphql::Value::from_json(json)?)
  }
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use anyhow::Result;
  use once_cell::sync::Lazy;
  use serde_json::json;

  use super::{ProtobufOperation, ProtobufService, ProtobufSet};

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

  #[test]
  fn unknown_file() -> Result<()> {
    let proto_file = get_test_file("_unknown.proto");
    let error = ProtobufSet::from_proto_file(&proto_file).unwrap_err();

    assert_eq!(
      error.to_string(),
      format!("Failed to parse proto file {}", proto_file.display())
    );

    Ok(())
  }

  #[test]
  fn service_not_found() -> Result<()> {
    let proto_file = get_test_file("greetings.proto");
    let file = ProtobufSet::from_proto_file(&proto_file)?;
    let error = ProtobufService::new(&file, "_unknown").unwrap_err();

    assert_eq!(error.to_string(), "Couldn't find definitions for service _unknown");

    Ok(())
  }

  #[test]
  fn method_not_found() -> Result<()> {
    let proto_file = get_test_file("greetings.proto");
    let file = ProtobufSet::from_proto_file(&proto_file)?;
    let service = ProtobufService::new(&file, "Greeter")?;
    let error = ProtobufOperation::new(&service, "_unknown").unwrap_err();

    assert_eq!(error.to_string(), "Could't find method _unknown");

    Ok(())
  }

  #[test]
  fn greetings_proto_file() -> Result<()> {
    let proto_file = get_test_file("greetings.proto");
    let file = ProtobufSet::from_proto_file(&proto_file)?;
    let service = ProtobufService::new(&file, "Greeter")?;
    let operation = ProtobufOperation::new(&service, "SayHello")?;

    let input = operation.convert_input(r#"{ "name": "hello message" }"#)?;

    assert_eq!(input, b"\0\0\0\0\x0f\n\rhello message");

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

  #[test]
  fn news_proto_file() -> Result<()> {
    let proto_file = get_test_file("news.proto");
    let file = ProtobufSet::from_proto_file(&proto_file)?;
    let service = ProtobufService::new(&file, "NewsService")?;
    let operation = ProtobufOperation::new(&service, "GetNews")?;

    let input = operation.convert_input(r#"{ "id": "1" }"#)?;

    assert_eq!(input, b"\0\0\0\0\x03\n\x011");

    let output = b"\0\0\0\0$\n\x011\x12\x06Note 1\x1a\tContent 1\"\x0cPost image 1";

    let parsed = operation.convert_output(output)?;

    assert_eq!(
      serde_json::to_value(parsed)?,
      json!({
        "id": "1", "title": "Note 1", "body": "Content 1", "postImage": "Post image 1"
      })
    );

    Ok(())
  }
}
