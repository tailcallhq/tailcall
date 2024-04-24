use std::collections::BTreeSet;

use derive_setters::Setters;
use prost_reflect::prost_types::{
    DescriptorProto, EnumDescriptorProto, FileDescriptorSet, ServiceDescriptorProto,
};

use crate::blueprint::GrpcMethod;
use crate::config::{Arg, Config, Field, Grpc, Tag, Type};
use crate::generator::GraphQLType;
use crate::valid::{Valid, ValidationError, Validator};

/// Assists in the mapping and retrieval of proto type names to custom formatted
/// strings based on the descriptor type.
#[derive(Setters)]
struct Context {
    /// The current proto package name.
    package: String,

    /// Final configuration that's being built up.
    config: Config,

    /// Root GraphQL query type
    query: String,
}

impl Context {
    fn new(query: &str) -> Self {
        Self {
            query: query.to_string(),
            package: Default::default(),
            config: Default::default(),
        }
    }

    /// Resolves the actual name and inserts the type.
    fn insert_type(mut self, name: String, ty: Type) -> Self {
        self.config.types.insert(name.to_string(), ty);
        self
    }

    /// Processes proto enum types.
    fn append_enums(mut self, enums: &Vec<EnumDescriptorProto>) -> Self {
        for enum_ in enums {
            let mut ty = Type::default();

            let enum_name = enum_.name();
            ty.tag = Some(Tag { id: enum_name.to_string() });

            let variants_result = Valid::from_iter(enum_.value.iter(), |v| {
                let graphql_type = GraphQLType::new(v.name());
                match graphql_type.as_enum_variant() {
                    Some(gt) => Valid::succeed(gt.to_string()),
                    None => Valid::fail_with(
                        format!(
                            "Error converting GraphQLType to enum variant: {:?}",
                            graphql_type
                        ),
                        "Invalid enum variant".to_string(),
                    ),
                }
            });

            if variants_result.is_succeed() {
                ty.variants = Some(
                    variants_result
                        .to_result()
                        .unwrap()
                        .into_iter()
                        .collect::<BTreeSet<String>>(),
                );
            }

            let type_name = match GraphQLType::new(enum_name).as_enum() {
                Some(enum_value) => Valid::succeed(enum_value.to_string()).to_result(),
                None => {
                    eprintln!("Error: Enum value not found");
                    Err(ValidationError::new("Enum value not found"))
                }
            }
            .unwrap_or_else(|err| {
                eprintln!("Error converting enum to string: {:?}", err);
                "DefaultTypeName".to_string()
            });
            self = self.insert_type(type_name, ty);
        }
        self
    }

    /// Processes proto message types.
    fn append_msg_type(mut self, messages: &Vec<DescriptorProto>) -> Self {
        for message in messages {
            let msg_name = message.name().to_string();
            if let Some(options) = message.options.as_ref() {
                if options.map_entry.unwrap_or_default() {
                    continue;
                }
            }

            self = self.append_enums(&message.enum_type);
            self = self.append_msg_type(&message.nested_type);

            let msg_type_result = match GraphQLType::new(&msg_name)
                .package(&self.package)
                .as_object_type()
            {
                Some(object_type) => Valid::succeed(object_type).to_result(),
                None => Err(ValidationError::new("Error: Unable to create object type")),
            };

            let msg_type = match msg_type_result {
                Ok(msg_type) => msg_type,
                Err(err) => {
                    eprintln!("Error: {:?}", err);
                    panic!("Failed to create object type");
                }
            };

            let mut ty = Type::default();
            for field in message.field.iter() {
                let field_name = match GraphQLType::new(field.name())
                    .package(&self.package)
                    .as_field()
                {
                    Some(field) => Valid::succeed(field),
                    None => Valid::fail_with("Error: Unable to create field", "Field is None"),
                };

                let field_name = field_name.to_result().unwrap_or_else(|err| {
                    eprintln!("Error: {:?}", err);
                    panic!("Failed to create field");
                });

                let mut cfg_field = Field::default();

                let label = field.label().as_str_name().to_lowercase();
                cfg_field.list = label.contains("repeated");
                cfg_field.required = label.contains("required") || cfg_field.list;

                if field.r#type.is_some() {
                    let type_of = convert_ty(field.r#type().as_str_name());
                    if type_of.eq("JSON") {
                        cfg_field.list = false;
                        cfg_field.required = false;
                    }
                    cfg_field.type_of = type_of;
                } else {
                    // for non-primitive types
                    let type_of = convert_ty(field.type_name());
                    let type_of = match GraphQLType::new(&type_of)
                        .package(self.package.as_str())
                        .as_object_type()
                    {
                        Some(object_type) => Valid::succeed(object_type.to_string()),
                        None => Valid::fail_with(
                            "Error: Unable to create object type",
                            "Object type is None",
                        ),
                    };

                    let type_of = type_of.to_result().unwrap_or_else(|err| {
                        eprintln!("Error: {:?}", err);
                        panic!("Failed to create object type");
                    });

                    cfg_field.type_of = type_of;
                }

                ty.fields.insert(field_name.to_string(), cfg_field);
            }

            ty.tag = Some(Tag { id: msg_type.id() });

            self = self.insert_type(msg_type.to_string(), ty);
        }
        self
    }

    /// Processes proto service definitions and their methods.
    fn append_query_service(mut self, services: &Vec<ServiceDescriptorProto>) -> Self {
        if services.is_empty() {
            return self;
        }

        let package = self.package.clone();
        let mut grpc_method = GrpcMethod { package, service: "".to_string(), name: "".to_string() };

        for service in services {
            let service_name = service.name().to_string();
            for method in &service.method {
                let field_name = match GraphQLType::new(method.name())
                    .package(&self.package)
                    .as_method()
                {
                    Some(method) => Valid::succeed(method),
                    None => Valid::fail_with("Error: Unable to create method", "Method is None"),
                };

                let field_name = field_name.to_result().unwrap_or_else(|err| {
                    eprintln!("Error: {:?}", err);
                    panic!("Failed to create method");
                });

                let mut cfg_field = Field::default();
                if let Some(arg_type) = get_input_ty(method.input_type()) {
                    let key = GraphQLType::new(&arg_type)
                        .package(&self.package)
                        .as_field()
                        .map(|field| field.to_string())
                        .unwrap_or_else(|| {
                            eprintln!("Error: Unable to create field for key");
                            panic!("Failed to create field for key");
                        });
                    let type_of = GraphQLType::new(&arg_type)
                        .package(&self.package)
                        .as_object_type()
                        .map(|object_type| object_type.to_string())
                        .unwrap_or_else(|| {
                            eprintln!("Error: Unable to create object type for type_of");
                            panic!("Failed to create object type for type_of");
                        });
                    let val = Arg {
                        type_of,
                        list: false,
                        required: true,
                        /* Setting it not null by default. There's no way to infer this
                         * from proto file */
                        doc: None,
                        modify: None,
                        default_value: None,
                    };

                    cfg_field.args.insert(key, val);
                }

                let output_ty = get_output_ty(method.output_type());
                let output_ty = match GraphQLType::new(&output_ty)
                    .package(&self.package)
                    .as_object_type()
                {
                    Some(object_type) => Valid::succeed(object_type.to_string()),
                    None => Valid::fail_with(
                        "Error: Unable to create object type",
                        "Object type is None",
                    ),
                };

                let output_ty = output_ty.to_result().unwrap_or_else(|err| {
                    eprintln!("Error: {:?}", err);
                    panic!("Failed to create object type");
                });

                cfg_field.type_of = output_ty;
                cfg_field.required = true;

                grpc_method.service = service_name.clone();
                grpc_method.name = field_name.to_string();

                cfg_field.grpc = Some(Grpc {
                    base_url: None,
                    body: None,
                    group_by: vec![],
                    headers: vec![],
                    method: field_name.id(),
                });

                let ty = self
                    .config
                    .types
                    .entry(self.query.clone())
                    .or_insert_with(|| {
                        self.config.schema.query = Some(self.query.clone());
                        Type::default()
                    });

                ty.fields.insert(field_name.to_string(), cfg_field);
            }
        }
        self
    }
}

/// Converts proto field types to a custom format.
fn convert_ty(proto_ty: &str) -> String {
    let binding = proto_ty.to_lowercase();
    let proto_ty = binding.strip_prefix("type_").unwrap_or(proto_ty);
    match proto_ty {
        "double" | "float" => "Float",
        "int32" | "int64" | "fixed32" | "fixed64" | "uint32" | "uint64" => "Int",
        "bool" => "Boolean",
        "string" | "bytes" => "String",
        "message" => "JSON", // JSON scalar is preloaded by tailcall, so there is no need to
        // explicitly define it in the config.
        x => x,
    }
    .to_string()
}

/// Determines the output type for a service method.
fn get_output_ty(output_ty: &str) -> String {
    // type, required
    match output_ty {
        "google.protobuf.Empty" => {
            // If it's no response is expected, we return an Empty scalar type
            "Empty".to_string()
        }
        any => {
            // Setting it not null by default. There's no way to infer this from proto file
            any.to_string()
        }
    }
}

fn get_input_ty(input_ty: &str) -> Option<String> {
    match input_ty {
        "google.protobuf.Empty" | "" => None,
        any => Some(any.to_string()),
    }
}

/// The main entry point that builds a Config object from proto descriptor sets.
pub fn from_proto(descriptor_sets: &[FileDescriptorSet], query: &str) -> Config {
    let mut ctx = Context::new(query);
    for descriptor_set in descriptor_sets.iter() {
        for file_descriptor in descriptor_set.file.iter() {
            ctx.package = file_descriptor.package().to_string();

            ctx = ctx
                .append_enums(&file_descriptor.enum_type)
                .append_msg_type(&file_descriptor.message_type)
                .append_query_service(&file_descriptor.service);
        }
    }

    ctx.config = ctx.config.remove_unused_types();

    ctx.config
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use prost_reflect::prost_types::{FileDescriptorProto, FileDescriptorSet};

    use crate::generator::from_proto::from_proto;

    fn get_proto_file_descriptor(name: &str) -> anyhow::Result<FileDescriptorProto> {
        let path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("src/generator/proto/{}", name));
        Ok(protox_parse::parse(
            name,
            std::fs::read_to_string(path)?.as_str(),
        )?)
    }

    fn new_file_desc(files: &[&str]) -> anyhow::Result<FileDescriptorSet> {
        let mut set = FileDescriptorSet::default();
        for file in files.iter() {
            let file = get_proto_file_descriptor(file)?;
            set.file.push(file);
        }
        Ok(set)
    }

    #[test]
    fn test_from_proto() -> anyhow::Result<()> {
        // news_enum.proto covers:
        // test for mutation
        // test for empty objects
        // test for optional type
        // test for enum
        // test for repeated fields
        // test for a type used as both input and output
        // test for two types having same name in different packages

        let set = new_file_desc(&["news.proto", "greetings_a.proto", "greetings_b.proto"])?;
        let result = from_proto(&[set], "Query").to_sdl();
        insta::assert_snapshot!(result);

        Ok(())
    }

    #[test]
    fn test_from_proto_no_pkg_file() -> anyhow::Result<()> {
        let set = new_file_desc(&["no_pkg.proto"])?;
        let result = from_proto(&[set], "Query").to_sdl();
        insta::assert_snapshot!(result);
        Ok(())
    }

    #[test]
    fn test_from_proto_no_service_file() -> anyhow::Result<()> {
        let set = new_file_desc(&["news_no_service.proto"])?;
        let result = from_proto(&[set], "Query").to_sdl();
        insta::assert_snapshot!(result);

        Ok(())
    }

    #[test]
    fn test_greetings_proto_file() {
        let set = new_file_desc(&["greetings.proto", "greetings_message.proto"]);

        let set = match set {
            Ok(desc) => desc,
            Err(err) => {
                eprintln!("Error: {:?}", err);
                panic!("Failed to create file descriptor set");
            }
        };
        let result = from_proto(&[set], "Query").to_sdl();
        insta::assert_snapshot!(result);
    }

    #[test]
    fn test_config_from_sdl() -> anyhow::Result<()> {
        let set = new_file_desc(&["news.proto", "greetings_a.proto", "greetings_b.proto"])?;

        let set1 = new_file_desc(&["news.proto"])?;
        let set2 = new_file_desc(&["greetings_a.proto"])?;
        let set3 = new_file_desc(&["greetings_b.proto"])?;

        let actual = from_proto(&[set.clone()], "Query").to_sdl();
        let expected = from_proto(&[set1, set2, set3], "Query").to_sdl();

        pretty_assertions::assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_required_types() -> anyhow::Result<()> {
        // required fields are deprecated in proto3 (https://protobuf.dev/programming-guides/dos-donts/#add-required)
        // this example uses proto2 to test the same.
        // for proto3 it's guaranteed to have a default value (https://protobuf.dev/programming-guides/proto3/#default)
        // and our implementation (https://github.com/tailcallhq/tailcall/pull/1537) supports default values by default.
        // so we do not need to explicitly mark fields as required.

        let set = new_file_desc(&["person.proto"])?;
        let config = from_proto(&[set], "Query").to_sdl();
        insta::assert_snapshot!(config);
        Ok(())
    }
}
