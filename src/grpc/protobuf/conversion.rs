use anyhow::Result;
use async_graphql::Value;
use protobuf::reflect::MessageDescriptor;
use protobuf::MessageDyn;

pub fn proto_to_value(message: &dyn MessageDyn) -> Result<Value> {
  // TODO: implement the conversion without intermediate conversion to string and serde_json
  // for references see:
  // - https://github.com/mrjones/rust-protobuf-json
  // - https://github.com/dflemstr/serde-protobuf
  let json = protobuf_json_mapping::print_to_string(message)?;
  let json = serde_json::from_str(&json)?;

  Ok(Value::from_json(json)?)
}

pub fn json_str_to_proto(message: &MessageDescriptor, json: &str) -> Result<Vec<u8>> {
  let message = protobuf_json_mapping::parse_dyn_from_str(message, json)?;

  Ok(message.write_to_bytes_dyn()?)
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use anyhow::{Context, Result};
  use protobuf::reflect::{FileDescriptor, MessageDescriptor};
  use protobuf_parse::Parser;
  use serde_json::json;

  use super::*;

  fn load_type_from_file(proto_file: &str, type_name: &str) -> Result<MessageDescriptor> {
    let root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut test_dir = root_dir.join(file!());
    test_dir.pop();
    test_dir.push("tests");

    let mut test_file = test_dir.clone();
    test_file.push(proto_file);

    let file_descriptor_protos = Parser::new()
      .pure()
      .include(test_dir)
      .input(test_file)
      .parse_and_typecheck()?;

    let file_descriptor_proto = file_descriptor_protos.file_descriptors.into_iter().next().unwrap();

    let file_descriptor = FileDescriptor::new_dynamic(file_descriptor_proto, &[])?;

    file_descriptor
      .message_by_package_relative_name(type_name)
      .with_context(|| format!("Failed to obtain type {}", type_name))
  }

  #[test]
  fn greetings_hello_request() -> Result<()> {
    let message_desc = load_type_from_file("greetings.proto", "HelloRequest")?;

    let proto_value = json_str_to_proto(&message_desc, r#"{ "name": "test" }"#)?;

    assert_eq!(proto_value, b"\n\x04test");

    let message = message_desc.parse_from_bytes(&proto_value)?;
    let value = proto_to_value(&*message)?;

    assert_eq!(
      serde_json::to_value(value)?,
      json!({
        "name": "test"
      })
    );

    Ok(())
  }

  #[test]
  fn news_news() -> Result<()> {
    let message_desc = load_type_from_file("news.proto", "News")?;

    let proto_value = json_str_to_proto(
      &message_desc,
      r#"{
      "id": "1", "title": "Note 1", "body": "Content 1", "postImage": "Post image 1"
    }"#,
    )?;

    assert_eq!(proto_value, b"\n\x011\x12\x06Note 1\x1a\tContent 1\"\x0cPost image 1");

    let message = message_desc.parse_from_bytes(&proto_value)?;
    let value = proto_to_value(&*message)?;

    assert_eq!(
      serde_json::to_value(value)?,
      json!({
        "id": "1", "title": "Note 1", "body": "Content 1", "postImage": "Post image 1"
      })
    );

    Ok(())
  }
}
