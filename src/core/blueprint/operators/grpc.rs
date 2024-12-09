use std::fmt::Display;

use prost_reflect::prost_types::FileDescriptorSet;
use prost_reflect::FieldDescriptor;
use tailcall_valid::{Valid, ValidationError, Validator};

use super::apply_select;
use crate::core::blueprint::BlueprintError;
use crate::core::config::group_by::GroupBy;
use crate::core::config::{Config, ConfigModule, Field, GraphQLOperationType, Grpc};
use crate::core::grpc::protobuf::{ProtobufOperation, ProtobufSet};
use crate::core::grpc::request_template::RequestTemplate;
use crate::core::helpers;
use crate::core::ir::model::{IO, IR};
use crate::core::json::JsonSchema;
use crate::core::mustache::Mustache;
use crate::core::worker_hooks::WorkerHooks;

fn to_url(grpc: &Grpc, method: &GrpcMethod) -> Valid<Mustache, String> {
    Valid::succeed(grpc.url.as_str()).and_then(|base_url| {
        let mut base_url = base_url.trim_end_matches('/').to_owned();
        base_url.push('/');
        base_url.push_str(format!("{}.{}", method.package, method.service).as_str());
        base_url.push('/');
        base_url.push_str(&method.name);

        helpers::url::to_url(&base_url)
    })
}

fn to_operation(
    method: &GrpcMethod,
    file_descriptor_set: FileDescriptorSet,
) -> Valid<ProtobufOperation, String> {
    Valid::from(
        ProtobufSet::from_proto_file(file_descriptor_set)
            .map_err(|e| ValidationError::new(e.to_string())),
    )
    .and_then(|set| {
        Valid::from(
            set.find_service(method)
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
    let field_schema = crate::core::blueprint::to_json_schema(&field.type_of, config);
    let args_schema = crate::core::blueprint::to_json_schema_for_args(&field.args, config);
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
) -> Valid<(), BlueprintError> {
    let input_type = &operation.input_type;
    let output_type = &operation.output_type;

    let input_type = match JsonSchema::try_from(input_type) {
        Ok(input_schema) => Valid::succeed(input_schema),
        Err(e) => Valid::from_validation_err(BlueprintError::from_validation_string(e)),
    };

    let output_type = match JsonSchema::try_from(output_type) {
        Ok(output_type) => Valid::succeed(output_type),
        Err(e) => Valid::from_validation_err(BlueprintError::from_validation_string(e)),
    };

    input_type
        .zip(output_type)
        .and_then(|(_input_schema, sub_type)| {
            // TODO: add validation for input schema - should compare result grpc.body to
            // schema
            let super_type = field_schema.field;
            // TODO: all of the fields in protobuf are optional actually
            // and if we want to mark some fields as required in GraphQL
            // JsonSchema won't match and the validation will fail
            match sub_type.is_a(&super_type, name).to_result() {
                Ok(res) => Valid::succeed(res),
                Err(e) => Valid::from_validation_err(BlueprintError::from_validation_string(e)),
            }
        })
}

fn validate_group_by(
    field_schema: &FieldSchema,
    operation: &ProtobufOperation,
    group_by: Vec<String>,
) -> Valid<(), BlueprintError> {
    let input_type = &operation.input_type;
    let output_type = &operation.output_type;
    let mut field_descriptor: Result<FieldDescriptor, ValidationError<BlueprintError>> = None
        .ok_or(ValidationError::new(BlueprintError::FieldNotFound(
            group_by[0].clone(),
        )));
    for item in group_by.iter().take(&group_by.len() - 1) {
        field_descriptor =
            output_type
                .get_field_by_json_name(item.as_str())
                .ok_or(ValidationError::new(BlueprintError::FieldNotFound(
                    item.clone(),
                )));
    }
    let output_type = field_descriptor
        .and_then(|f| JsonSchema::try_from(&f).map_err(BlueprintError::from_validation_string));

    let json_schema = match JsonSchema::try_from(input_type) {
        Ok(schema) => Valid::succeed(schema),
        Err(e) => Valid::from_validation_err(BlueprintError::from_validation_string(e)),
    };

    json_schema
        .zip(Valid::from(output_type))
        .and_then(|(_input_schema, output_schema)| {
            // TODO: add validation for input schema - should compare result grpc.body to
            // schema considering repeated message type
            let fields = &field_schema.field;
            // we're treating List types for gRPC as optional.
            let fields = JsonSchema::Opt(Box::new(JsonSchema::Arr(Box::new(fields.to_owned()))));
            match fields
                .is_a(&output_schema, group_by[0].as_str())
                .to_result()
            {
                Ok(res) => Valid::succeed(res),
                Err(e) => Valid::from_validation_err(BlueprintError::from_validation_string(e)),
            }
        })
}

pub struct CompileGrpc<'a> {
    pub config_module: &'a ConfigModule,
    pub operation_type: &'a GraphQLOperationType,
    pub field: &'a Field,
    pub grpc: &'a Grpc,
    pub validate_with_schema: bool,
}
pub struct GrpcMethod {
    pub package: String,
    pub service: String,
    pub name: String,
}

impl Display for GrpcMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.package, self.service, self.name)
    }
}

impl TryFrom<&str> for GrpcMethod {
    type Error = ValidationError<crate::core::blueprint::BlueprintError>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = value.rsplitn(3, '.').collect();
        match &parts[..] {
            &[name, service, id] => {
                let method = GrpcMethod {
                    package: id.to_owned(),
                    service: service.to_owned(),
                    name: name.to_owned(),
                };
                Ok(method)
            }
            _ => Err(ValidationError::new(
                BlueprintError::InvalidGrpcMethodFormat(value.to_string()),
            )),
        }
    }
}

pub fn compile_grpc(inputs: CompileGrpc) -> Valid<IR, BlueprintError> {
    let config_module = inputs.config_module;
    let operation_type = inputs.operation_type;
    let field = inputs.field;
    let grpc = inputs.grpc;
    let validate_with_schema = inputs.validate_with_schema;
    let dedupe = grpc.dedupe.unwrap_or_default();

    Valid::from(GrpcMethod::try_from(grpc.method.as_str()))
        .and_then(|method| {
            let file_descriptor_set = config_module.extensions().get_file_descriptor_set();

            if file_descriptor_set.file.is_empty() {
                return Valid::fail(BlueprintError::ProtobufFilesNotSpecifiedInConfig);
            }

            match to_operation(&method, file_descriptor_set)
                .fuse(to_url(grpc, &method))
                .fuse(helpers::headers::to_mustache_headers(&grpc.headers))
                .fuse(helpers::body::to_body(grpc.body.as_ref()))
                .to_result()
            {
                Ok(data) => Valid::succeed(data),
                Err(e) => Valid::from_validation_err(BlueprintError::from_validation_string(e)),
            }
        })
        .and_then(|(operation, url, headers, body)| {
            let validation = if validate_with_schema {
                let field_schema = json_schema_from_field(config_module, field);
                if grpc.batch_key.is_empty() {
                    validate_schema(field_schema, &operation, field.type_of.name()).unit()
                } else {
                    validate_group_by(&field_schema, &operation, grpc.batch_key.clone()).unit()
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
            let on_response = grpc.on_response_body.clone();
            let hook = WorkerHooks::try_new(None, on_response).ok();

            let io = if !grpc.batch_key.is_empty() {
                IR::IO(IO::Grpc {
                    req_template,
                    group_by: Some(GroupBy::new(grpc.batch_key.clone(), None)),
                    dl_id: None,
                    dedupe,
                    hook,
                })
            } else {
                IR::IO(IO::Grpc { req_template, group_by: None, dl_id: None, dedupe, hook })
            };

            (io, &grpc.select)
        })
        .and_then(apply_select)
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use tailcall_valid::ValidationError;

    use super::GrpcMethod;
    use crate::core::blueprint::BlueprintError;

    #[test]
    fn try_from_grpc_method() {
        let method = GrpcMethod::try_from("package_name.ServiceName.MethodName").unwrap();
        let method1 = GrpcMethod::try_from("package.name.ServiceName.MethodName").unwrap();

        assert_eq!(method.package, "package_name");
        assert_eq!(method.service, "ServiceName");
        assert_eq!(method.name, "MethodName");

        assert_eq!(method1.package, "package.name");
        assert_eq!(method1.service, "ServiceName");
        assert_eq!(method1.name, "MethodName");
    }

    #[test]
    fn try_from_grpc_method_invalid() {
        let result = GrpcMethod::try_from("package_name.ServiceName");

        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            ValidationError::new(BlueprintError::InvalidGrpcMethodFormat(
                "package_name.ServiceName".to_string()
            ))
        );
    }
}
