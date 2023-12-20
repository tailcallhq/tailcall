use std::path::Path;

use prost_reflect::Kind;

use crate::blueprint::{FieldDefinition, TypeLike};
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

fn generate_json_schema(
  msg_descriptor: &prost_reflect::MessageDescriptor,
) -> Result<JsonSchema, ValidationError<String>> {
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
  match a {
    JsonSchema::Obj(a) => {
      if let JsonSchema::Obj(b) = b {
        return Valid::from_iter(b.iter(), |(key, b)| {
          Valid::from_option(a.get(key), format!("missing key: {}", key)).and_then(|a| compare_json_schema(a, b, key))
        })
        .trace(name)
        .unit();
      } else {
        return Valid::fail("expected Object type".to_string()).trace(name);
      }
    }
    JsonSchema::Arr(a) => {
      if let JsonSchema::Arr(b) = b {
        return compare_json_schema(a, b, name);
      } else {
        return Valid::fail("expected Array type".to_string()).trace(name);
      }
    }
    JsonSchema::Opt(a) => {
      if let JsonSchema::Opt(b) = b {
        compare_json_schema(a, b, name).unit();
      } else {
        return Valid::fail("expected type to be optional".to_string()).trace(name);
      }
    }
    JsonSchema::Str => {
      if b != a {
        return Valid::fail(format!("expected String, got {:?}", b)).trace(name);
      }
    }
    JsonSchema::Num => {
      if b != a {
        return Valid::fail(format!("expected Number, got {:?}", b)).trace(name);
      }
    }
    JsonSchema::Bool => {
      if b != a {
        return Valid::fail(format!("expected Boolean, got {:?}", b)).trace(name);
      }
    }
  }
  Valid::succeed(())
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

fn json_schema_from_field(config: &Config, field: &Field) -> FieldSchema {
  let field_schema = crate::blueprint::to_json_schema_for_field(field, config);
  let args_schema = crate::blueprint::to_json_schema_for_args(&field.args, config);
  FieldSchema { args: args_schema, field: field_schema }
}
pub struct FieldSchema {
  pub args: JsonSchema,
  pub field: JsonSchema,
}
fn validate_schema(field_schema: FieldSchema, operation: &ProtobufOperation, name: &str) -> Valid<(), String> {
  let input_type = &operation.input_type;
  let output_type = &operation.output_type;

  Valid::from(generate_json_schema(input_type))
    .zip(Valid::from(generate_json_schema(output_type)))
    .and_then(|(_input_schema, output_schema)| {
      let fields = field_schema.field;
      let _args = field_schema.args;
      compare_json_schema(&fields, &output_schema, name)
    })
}

pub fn update_grpc<'a>(
  operation_type: &'a GraphQLOperationType,
) -> TryFold<'a, (&'a Config, &'a Field, &'a config::Type, &'a str), FieldDefinition, String> {
  TryFold::<(&Config, &Field, &config::Type, &'a str), FieldDefinition, String>::new(
    |(config, field, _type_of, _name), b_field| {
      let Some(grpc) = &field.grpc else {
        return Valid::succeed(b_field);
      };

      to_url(grpc, config)
        .zip(to_operations(grpc))
        .zip(helpers::headers::to_headervec(&grpc.headers))
        .zip(helpers::body::to_body(grpc.body.as_deref()))
        .and_then(|(((url, (operation, batch)), headers), body)| {
          let field_schema = json_schema_from_field(config, field);
          validate_schema(field_schema, &operation, field.name()).and_then(|_| {
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
