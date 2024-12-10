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
) -> Valid<(), String> {
    let input_type = &operation.input_type;
    let output_type = &operation.output_type;

    Valid::from(JsonSchema::try_from(input_type))
        .zip(Valid::from(JsonSchema::try_from(output_type)))
        .and_then(|(input_schema, output_schema)| {
            let fields = &field_schema.field;
            let args = &field_schema.args;

            // Treat repeated message types as optional in input schema
            let normalized_input_schema = normalize_repeated_types(&input_schema);

            // Validate input schema against args
            args.compare(&normalized_input_schema, &format!("Input validation failed for {}", name))?;

            // Validate output schema against fields
            fields.compare(&output_schema, &format!("Output validation failed for {}", name))
        })
}
fn normalize_repeated_types(schema: &JsonSchema) -> JsonSchema {
    match schema {
        JsonSchema::Arr(inner_schema) => {
            // Treat repeated types (arrays) as optional
            JsonSchema::Optional(Box::new(inner_schema.clone()))
        }
        JsonSchema::Object(fields) => {
            let normalized_fields = fields
                .iter()
                .map(|(key, value)| (key.clone(), normalize_repeated_types(value)))
                .collect();
            JsonSchema::Object(normalized_fields)
        }
        _ => schema.clone(),
    }
}
fn validate_group_by(
    field_schema: &FieldSchema,
    operation: &ProtobufOperation,
    group_by: Vec<String>,
) -> Valid<(), String> {
    let input_type = &operation.input_type;
    let output_type = &operation.output_type;

    let input_schema = JsonSchema::try_from(input_type)?;
    let output_schema = JsonSchema::try_from(output_type)?;

    let normalized_input_schema = normalize_repeated_types(&input_schema);

    let fields = JsonSchema::Arr(Box::new(field_schema.field.to_owned()));
    let args = JsonSchema::Arr(Box::new(field_schema.args.to_owned()));

    args.compare(
        &normalized_input_schema,
        &format!("Input validation failed for group_by {:?}", group_by),
    )?;
    fields.compare(
        &output_schema,
        &format!("Output validation failed for group_by {:?}", group_by),
    )
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
            return Valid::fail("Protobuf files were not specified in the config".to_string());
        }

        to_operation(&method, file_descriptor_set)
            .fuse(to_url(grpc, &method, config_module))
            .fuse(helpers::headers::to_mustache_headers(&grpc.headers))
            .fuse(helpers::body::to_body(grpc.body.as_ref()))
            .into()
    })
    .and_then(|(operation, url, headers, body)| {
        let validation = if validate_with_schema {
            let field_schema = json_schema_from_field(config_module, field);
            if grpc.batch_key.is_empty() {
                // Add input validation with repeated type normalization
                validate_schema(field_schema, &operation, field.name()).unit()
            } else {
                validate_group_by(&field_schema, &operation, grpc.batch_key.clone()).unit()
            }
        } else {
            Valid::succeed(())
        };
        validation.map(|_| (url, headers, operation, body))
    })

}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use tailcall_valid::ValidationError;

    use super::GrpcMethod;
    use crate::core::blueprint::BlueprintError;
    #[test]
fn validate_repeated_types_as_optional() {
    let operation = ProtobufOperation {
        input_type: "RepeatedInputType".to_string(),
        output_type: "ValidOutputType".to_string(),
    };

    let field_schema = FieldSchema {
        args: JsonSchema::Arr(Box::new(JsonSchema::String)),
        field: JsonSchema::Object(HashMap::new()),
    };

    let result = validate_schema(field_schema, &operation, "test_operation");
    assert!(result.is_ok());
}


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
fn grpc_repeated_types_validation_integration() {
    let config_module = MockConfigModule::new();
    let operation_type = GraphQLOperationType::Query;
    let field = Field::new("test_field", "RepeatedInputType");

    let grpc = Grpc {
        method: "package.Service.Method".to_string(),
        base_url: Some("http://localhost:5000".to_string()),
        headers: None,
        body: Some(vec!["repeated_field"]),
        batch_key: vec![],
    };

    let compile_inputs = CompileGrpc {
        config_module: &config_module,
        operation_type: &operation_type,
        field: &field,
        grpc: &grpc,
        validate_with_schema: true,
    };

    let result = compile_grpc(compile_inputs);
    assert!(result.is_ok());
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
