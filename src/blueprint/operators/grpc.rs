use prost_reflect::prost_types::FileDescriptorSet;
use prost_reflect::FieldDescriptor;

use crate::blueprint::{FieldDefinition, TypeLike};
use crate::config::group_by::GroupBy;
use crate::config::{Config, ConfigSet, Field, GraphQLOperationType, Grpc};
use crate::grpc::protobuf::{ProtobufOperation, ProtobufSet};
use crate::grpc::request_template::RequestTemplate;
use crate::json::JsonSchema;
use crate::lambda::{Expression, IO};
use crate::mustache::Mustache;
use crate::try_fold::TryFold;
use crate::valid::{Valid, ValidationError, Validator};
use crate::{config, helpers};

fn to_url(grpc: &Grpc, service: &str, method: &str, config: &Config) -> Valid<Mustache, String> {
    Valid::from_option(
        grpc.base_url.as_ref().or(config.upstream.base_url.as_ref()),
        "No base URL defined".to_string(),
    )
    .and_then(|base_url| {
        let mut base_url = base_url.trim_end_matches('/').to_owned();
        base_url.push('/');
        base_url.push_str(service);
        base_url.push('/');
        base_url.push_str(method);

        helpers::url::to_url(&base_url)
    })
}

fn to_operation(
    service: &str,
    method: &str,
    file_descriptor_set: &FileDescriptorSet,
) -> Valid<ProtobufOperation, String> {
    println!("service: {}", service);
    Valid::from(
        ProtobufSet::from_proto_file(file_descriptor_set)
            .map_err(|e| ValidationError::new(e.to_string())),
    )
    .and_then(|set| {
        Valid::from(
            set.find_service(service)
                .map_err(|e| ValidationError::new(e.to_string())),
        )
    })
    .and_then(|service| {
        Valid::from(
            service
                .find_operation(method)
                .map_err(|e| ValidationError::new(e.to_string())),
        )
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
fn validate_schema(
    field_schema: FieldSchema,
    operation: &ProtobufOperation,
    name: &str,
) -> Valid<(), String> {
    let input_type = &operation.input_type;
    let output_type = &operation.output_type;

    Valid::from(JsonSchema::try_from(input_type))
        .zip(Valid::from(JsonSchema::try_from(output_type)))
        .and_then(|(_input_schema, output_schema)| {
            // TODO: add validation for input schema - should compare result grpc.body to schema
            let fields = field_schema.field;
            let _args = field_schema.args;
            // TODO: all of the fields in protobuf are optional actually
            // and if we want to mark some fields as required in GraphQL
            // JsonSchema won't match and the validation will fail
            fields.compare(&output_schema, name)
        })
}

fn validate_group_by(
    field_schema: &FieldSchema,
    operation: &ProtobufOperation,
    group_by: Vec<String>,
) -> Valid<(), String> {
    let input_type = &operation.input_type;
    let output_type = &operation.output_type;
    let mut field_descriptor: Result<FieldDescriptor, ValidationError<String>> = None.ok_or(
        ValidationError::new(format!("field {} not found", group_by[0])),
    );
    for item in group_by.iter().take(&group_by.len() - 1) {
        field_descriptor = output_type
            .get_field_by_json_name(item.as_str())
            .ok_or(ValidationError::new(format!("field {} not found", item)));
    }
    let output_type = field_descriptor.and_then(|f| JsonSchema::try_from(&f));

    Valid::from(JsonSchema::try_from(input_type))
        .zip(Valid::from(output_type))
        .and_then(|(_input_schema, output_schema)| {
            // TODO: add validation for input schema - should compare result grpc.body to schema considering repeated message type
            let fields = &field_schema.field;
            let args = &field_schema.args;
            let fields = JsonSchema::Arr(Box::new(fields.to_owned()));
            let _args = JsonSchema::Arr(Box::new(args.to_owned()));
            fields.compare(&output_schema, group_by[0].as_str())
        })
}

pub struct CompileGrpc<'a> {
    pub config_set: &'a ConfigSet,
    pub operation_type: &'a GraphQLOperationType,
    pub field: &'a Field,
    pub grpc: &'a Grpc,
    pub validate_with_schema: bool,
}

pub fn compile_grpc(inputs: CompileGrpc) -> Valid<Expression, String> {
    let config_set = inputs.config_set;
    let operation_type = inputs.operation_type;
    let field = inputs.field;
    let grpc = inputs.grpc;
    let validate_with_schema = inputs.validate_with_schema;

    // all data is on grpc.method
    // I need to split it
    // method[0] is proto_id
    // proto_id.method[1] is service
    // method[2] is method

    dbg!(grpc.method.clone());

    let method: Vec<&str> = grpc.method.split('.').collect();

    let proto_id = method[0];
    let service = format!("{}.{}", proto_id, method[1]);
    let method = method[2];

    println!("proto_id: {}", proto_id);
    println!("service: {}", service);
    println!("method: {}", method);

    Valid::from_option(
        config_set.extensions.get_file_descriptor(&proto_id),
        format!("File descriptor not found for proto id: {}", proto_id),
    )
    .and_then(|file_descriptor_set| to_operation(&service, method, file_descriptor_set))
    .fuse(to_url(grpc, &service, method, config_set))
    .fuse(helpers::headers::to_mustache_headers(&grpc.headers))
    .fuse(helpers::body::to_body(grpc.body.as_deref()))
    .and_then(|(operation, url, headers, body)| {
        let validation = if validate_with_schema {
            let field_schema = json_schema_from_field(config_set, field);
            if grpc.group_by.is_empty() {
                validate_schema(field_schema, &operation, field.name()).unit()
            } else {
                validate_group_by(&field_schema, &operation, grpc.group_by.clone()).unit()
            }
        } else {
            Valid::succeed(())
        };
        validation.map(|_| (url, headers, operation, body))
    })
    .map(|(url, headers, operation, body)| {
        let req_template = RequestTemplate {
            url,
            headers,
            operation,
            body,
            operation_type: operation_type.clone(),
        };
        if !grpc.group_by.is_empty() {
            Expression::IO(IO::Grpc {
                req_template,
                group_by: Some(GroupBy::new(grpc.group_by.clone())),
                dl_id: None,
            })
        } else {
            Expression::IO(IO::Grpc { req_template, group_by: None, dl_id: None })
        }
    })
}

pub fn update_grpc<'a>(
    operation_type: &'a GraphQLOperationType,
) -> TryFold<'a, (&'a ConfigSet, &'a Field, &'a config::Type, &'a str), FieldDefinition, String> {
    TryFold::<(&ConfigSet, &Field, &config::Type, &'a str), FieldDefinition, String>::new(
        |(config_set, field, type_of, _name), b_field| {
            let Some(grpc) = &field.grpc else {
                return Valid::succeed(b_field);
            };

            compile_grpc(CompileGrpc {
                config_set,
                operation_type,
                field,
                grpc,
                validate_with_schema: true,
            })
            .map(|resolver| b_field.resolver(Some(resolver)))
            .and_then(|b_field| b_field.validate_field(type_of, config_set).map_to(b_field))
        },
    )
}
