use derive_setters::Setters;
use prost_reflect::prost_types::{
    DescriptorProto, EnumDescriptorProto, FileDescriptorSet, ServiceDescriptorProto,
};

use crate::core::blueprint::GrpcMethod;
use crate::core::config::{Arg, Config, Enum, Field, Grpc, Tag, Type};
use crate::core::generator::GraphQLType;
use crate::core::getter::Getter;

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
    fn insert_type(mut self, ty: Type) -> Self {
        self.config.types.push(ty);
        self
    }

    /// Processes proto enum types.
    fn append_enums(mut self, enums: &Vec<EnumDescriptorProto>) -> Self {
        for enum_ in enums {
            let enum_name = enum_.name();

            let variants = enum_
                .value
                .iter()
                .map(|v| {
                    GraphQLType::new(v.name())
                        .as_enum_variant()
                        .unwrap()
                        .to_string()
                })
                .collect::<Vec<String>>();

            let type_name = GraphQLType::new(enum_name)
                .package(&self.package)
                .as_enum()
                .unwrap()
                .to_string();
            self.config.enums =
                self.config
                    .enums
                    .insert(Enum { name: type_name, variants, doc: None });
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

            let msg_type = GraphQLType::new(&msg_name)
                .package(&self.package)
                .as_object_type()
                .unwrap();

            let mut ty = Type::new(msg_type.to_string());
            for field in message.field.iter() {
                let field_name = GraphQLType::new(field.name())
                    .package(&self.package)
                    .as_field()
                    .unwrap();

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
                    let type_of = GraphQLType::new(&type_of)
                        .package(self.package.as_str())
                        .as_object_type()
                        .unwrap()
                        .to_string();

                    cfg_field.type_of = type_of;
                }

                ty.fields.insert(field_name.to_string(), cfg_field);
            }

            ty.tag = Some(Tag { id: msg_type.id() });

            self = self.insert_type(ty);
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
                let field_name = GraphQLType::new(method.name())
                    .package(&self.package)
                    .as_method()
                    .unwrap();

                let mut cfg_field = Field::default();
                if let Some(arg_type) = get_input_ty(method.input_type()) {
                    let key = GraphQLType::new(&arg_type)
                        .package(&self.package)
                        .as_field()
                        .unwrap()
                        .to_string();
                    let type_of = GraphQLType::new(&arg_type)
                        .package(&self.package)
                        .as_object_type()
                        .unwrap()
                        .to_string();
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
                let output_ty = GraphQLType::new(&output_ty)
                    .package(&self.package)
                    .as_object_type()
                    .unwrap()
                    .to_string();
                cfg_field.type_of = output_ty;
                cfg_field.required = true;

                grpc_method.service.clone_from(&service_name);
                grpc_method.name = field_name.to_string();

                cfg_field.grpc = Some(Grpc {
                    base_url: None,
                    body: None,
                    group_by: vec![],
                    headers: vec![],
                    method: field_name.id(),
                });

                if let Some(ty) = self.config.types.get_mut(&self.query) {
                    ty.fields.insert(field_name.to_string(), cfg_field);
                } else {
                    let mut ty = Type::new(&self.query);
                    ty.fields.insert(field_name.to_string(), cfg_field);
                    self.config.schema.query = Some(self.query.clone());
                    self.config.types.push(ty);
                }
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

    let unused_types = ctx.config.unused_types();
    ctx.config = ctx.config.remove_types(unused_types);

    ctx.config
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use prost_reflect::prost_types::{FileDescriptorProto, FileDescriptorSet};

    use crate::core::generator::from_proto::from_proto;

    fn get_proto_file_descriptor(name: &str) -> anyhow::Result<FileDescriptorProto> {
        let path = PathBuf::from(tailcall_fixtures::generator::proto::SELF).join(name);
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
        let set = new_file_desc(&["greetings.proto", "greetings_message.proto"]).unwrap();
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
