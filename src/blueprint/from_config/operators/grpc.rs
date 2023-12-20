use std::path::Path;

use prost_reflect::Kind;

use crate::blueprint::FieldDefinition;
use crate::config::group_by::GroupBy;
use crate::config::{Config, Field, GraphQLOperationType, Grpc, GrpcBatchOperation};
use crate::grpc::protobuf::{ProtobufOperation, ProtobufSet};
use crate::grpc::request_template::RequestTemplate;
use crate::json::JsonSchema;
use crate::lambda::Lambda;
use crate::mustache::Mustache;
use crate::try_fold::TryFold;
use crate::valid::{Valid, ValidationError};
use crate::{config, helpers};

fn generate_json_schema(msg_descriptor: &prost_reflect::MessageDescriptor) -> anyhow::Result<JsonSchema> {
  let mut map = std::collections::HashMap::new();
  let fields = msg_descriptor.fields();

  for field in fields {
    let field_schema = match field.kind() {
      Kind::Double => JsonSchema::Num,
      Kind::Float => JsonSchema::Num,
      Kind::Int32 => JsonSchema::Num,
      Kind::Int64 => JsonSchema::Num,
      Kind::Uint32 => JsonSchema::Num,
      Kind::Uint64 => JsonSchema::Num,
      Kind::Sint32 => JsonSchema::Num,
      Kind::Sint64 => JsonSchema::Num,
      Kind::Fixed32 => JsonSchema::Num,
      Kind::Fixed64 => JsonSchema::Num,
      Kind::Sfixed32 => JsonSchema::Num,
      Kind::Sfixed64 => JsonSchema::Num,
      Kind::Bool => JsonSchema::Bool,
      Kind::String => JsonSchema::Str,
      Kind::Bytes => JsonSchema::Str,
      Kind::Message(msg) => generate_json_schema(&msg)?,
      Kind::Enum(_) => {
        todo!("Enum")
      }
    };
    let field_schema = if field.cardinality().eq(&prost_reflect::Cardinality::Optional) {
      JsonSchema::Opt(Box::new(field_schema))
    } else {
      field_schema
    };
    let field_schema = if field.is_list() {
      JsonSchema::Arr(Box::new(field_schema))
    } else {
      field_schema
    };

    map.insert(field.name().to_string(), field_schema);
  }

  Ok(JsonSchema::Obj(map))
}
fn compare_json_schema(a: &JsonSchema, b: &JsonSchema, name: &str) -> Valid<(), String> {
  match (a, b) {
    (JsonSchema::Str, JsonSchema::Str) => Valid::succeed(()),
    (JsonSchema::Num, JsonSchema::Num) => Valid::succeed(()),
    (JsonSchema::Bool, JsonSchema::Bool) => Valid::succeed(()),
    (JsonSchema::Arr(a), JsonSchema::Arr(b)) => compare_json_schema(a, b, name),
    (JsonSchema::Opt(a), JsonSchema::Opt(b)) => compare_json_schema(a, b, name),
    (JsonSchema::Obj(a), JsonSchema::Obj(b)) => Valid::from_iter(a.iter(), |(key, a)| {
      Valid::from_option(b.get(key), format!("missing key: {}", key)).and_then(|b| compare_json_schema(a, b, key))
    })
    .unit(),
    _ => Valid::fail(format!("expected {:?}, got {:?}", a, b)).trace(name),
  }
}
fn to_url(grpc: &Grpc, config: &Config) -> Valid<Mustache, String> {
  Valid::from_option(
    grpc.base_url.as_ref().or(config.upstream.base_url.as_ref()),
    "No base URL defined".to_string(),
  )
  .and_then(|base_url| {
    let mut base_url = base_url.trim_end_matches('/').to_owned();
    base_url.push('/');
    base_url.push_str(&grpc.service);
    base_url.push('/');
    base_url.push_str(&grpc.method);

    helpers::url::to_url(&base_url)
  })
}

fn to_operations(grpc: &Grpc) -> Valid<(ProtobufOperation, Option<GrpcBatchOperation>), String> {
  Valid::from(
    ProtobufSet::from_proto_file(Path::new(&grpc.proto_path)).map_err(|e| ValidationError::new(e.to_string())),
  )
  .and_then(|set| {
    Valid::from(
      set
        .find_service(&grpc.service)
        .map_err(|e| ValidationError::new(e.to_string())),
    )
  })
  .and_then(|service| {
    let operation = Valid::from(
      service
        .find_operation(&grpc.method)
        .map_err(|e| ValidationError::new(e.to_string())),
    );

    if let Some(batch) = &grpc.batch {
      return operation.zip(Valid::from(
        service
          .find_operation(&batch.method)
          .map(|operation| Some(GrpcBatchOperation { operation, group_by: GroupBy::new(batch.group_by.clone()) }))
          .map_err(|e| ValidationError::new(e.to_string())),
      ));
    }

    operation.zip(Valid::succeed(None))
  })
}

fn validate_schema(config: &Config, field: &Field, operation: &ProtobufOperation, name: &str) -> Valid<(), String> {
  let input_type = &operation.input_type;
  let output_type = &operation.output_type;

  let _input_schema = generate_json_schema(input_type).unwrap();
  let output_schema = generate_json_schema(output_type).unwrap();

  let field_schema = crate::blueprint::to_json_schema_for_field(field, config);
  let _args_schema = crate::blueprint::to_json_schema_for_args(&field.args, config);
  compare_json_schema(&output_schema, &field_schema, name)
}

pub fn update_grpc<'a>(
  operation_type: &'a GraphQLOperationType,
) -> TryFold<'a, (&'a Config, &'a Field, &'a config::Type, &'a str), FieldDefinition, String> {
  TryFold::<(&Config, &Field, &config::Type, &'a str), FieldDefinition, String>::new(
    |(config, field, _type_of, name), b_field| {
      let Some(grpc) = &field.grpc else {
        return Valid::succeed(b_field);
      };

      to_url(grpc, config)
        .zip(to_operations(grpc))
        .zip(helpers::headers::to_headervec(&grpc.headers))
        .zip(helpers::body::to_body(grpc.body.as_deref()))
        .and_then(|(((url, (operation, batch)), headers), body)| {
          validate_schema(config, field, &operation, name).and_then(|_| {
            let request_template =
              RequestTemplate { url, headers, operation, body, operation_type: operation_type.to_owned() };

            Valid::succeed(b_field.resolver(Some(
              Lambda::from_grpc_request_template(request_template, batch).expression,
            )))
          })
        })
    },
  )
}
