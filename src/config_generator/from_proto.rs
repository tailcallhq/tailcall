#![allow(unused)]

use std::collections::{BTreeSet, HashMap};

use convert_case::{Case, Casing};
use derive_setters::Setters;
use prost_reflect::prost_types::{
    DescriptorProto, EnumDescriptorProto, FileDescriptorSet, ServiceDescriptorProto,
};
use strum_macros::Display;

use crate::blueprint::GrpcMethod;
use crate::config::{Arg, Config, Field, Grpc, Tag, Type};

pub(super) static DEFAULT_SEPARATOR: &str = "__";

/// Enum to represent the type of the descriptor
#[derive(Display, Clone)]
enum DescriptorType {
    Enum,
    Message,
    Operation,
}

impl DescriptorType {
    fn as_str_name(&self, package: &str, name: &str) -> String {
        match self {
            DescriptorType::Enum => {
                format!("{}{}{}", package, DEFAULT_SEPARATOR, name)
            }
            DescriptorType::Message => {
                format!("{}{}{}", package, DEFAULT_SEPARATOR, name)
            }
            DescriptorType::Operation => format!(
                "{}{}{}",
                package.to_case(Case::Camel),
                DEFAULT_SEPARATOR,
                name.to_case(Case::Camel),
            ),
        }
    }
}

/// Assists in the mapping and retrieval of proto type names to custom formatted
/// strings based on the descriptor type.
#[derive(Setters)]
struct Context {
    /// Maps proto type names to custom formatted names.
    map: HashMap<String, String>,

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
            map: Default::default(),
            package: Default::default(),
            config: Default::default(),
        }
    }

    /// Formats a proto type name based on its `DescriptorType`.
    fn get_name(&self, name: &str, ty: DescriptorType) -> String {
        let package = self
            .package
            .replace('.', DEFAULT_SEPARATOR)
            .to_case(Case::UpperCamel);

        ty.as_str_name(&package, name)
    }

    /// Inserts a formatted name into the map.
    fn insert(mut self, name: &str, ty: DescriptorType) -> Self {
        self.map.insert(
            format!("{}.{}", self.package, name),
            self.get_name(name, ty),
        );
        self
    }
    /// Retrieves a formatted name from the map.
    fn get(&self, name: &str) -> Option<String> {
        self.map.get(&format!("{}.{}", self.package, name)).cloned()
    }

    /// Retrieves or creates a Type configuration for a given proto type.
    fn get_ty(&self, name: &str) -> Type {
        let mut ty = self
            .config
            .types
            .get(
                &self
                    .get(name)
                    .unwrap_or_else(|| panic!("Expected key not found in types map: {}", name)),
            )
            .cloned()
            .unwrap_or_default(); // it should be
                                  // safe to call
                                  // unwrap here
        ty.tag = Some(Tag { id: format!("{}.{}", self.package, name) });

        ty
    }

    /// Processes proto enum types.
    fn append_enums(mut self, enums: Vec<EnumDescriptorProto>) -> Self {
        for enum_ in enums {
            let enum_name = enum_.name();

            self = self.insert(enum_name, DescriptorType::Enum);
            let mut ty = self.get_ty(enum_name);

            let mut variants = enum_
                .value
                .iter()
                .map(|v| v.name().to_string())
                .collect::<BTreeSet<String>>();
            if let Some(vars) = ty.variants {
                variants.extend(vars);
            }
            ty.variants = Some(variants);
            self.config.types.insert(
                self.get(enum_name).unwrap_or_else(|| {
                    panic!("Expected key not found in types map: {}", enum_name)
                }),
                ty,
            );
            // it should be
            // safe to call
            // unwrap here
        }
        self
    }

    /// Processes proto message types.
    fn append_msg_type(mut self, messages: Vec<DescriptorProto>) -> Self {
        if messages.is_empty() {
            return self;
        }
        for message in messages {
            let msg_name = message.name().to_string();

            self = self.insert(&msg_name, DescriptorType::Message);
            let mut ty = self.get_ty(&msg_name);

            self = self.append_enums(message.enum_type);
            self = self.append_msg_type(message.nested_type);

            for field in message.field {
                let field_name = field.name().to_string();
                let mut cfg_field = Field::default();

                let label = field.label().as_str_name().to_lowercase();
                cfg_field.list = label.contains("repeated");
                cfg_field.required = label.contains("required");

                if field.r#type.is_some() {
                    let type_of = convert_ty(field.r#type().as_str_name());
                    cfg_field.type_of = type_of.to_string();
                } else {
                    // for non-primitive types
                    let type_of = convert_ty(field.type_name());
                    cfg_field.type_of = self.get(&type_of).unwrap_or(type_of);
                }

                ty.fields.insert(field_name, cfg_field);
            }

            self.config.types.insert(
                self.get(&msg_name)
                    .unwrap_or_else(|| panic!("Expected key not found in types map: {}", msg_name)),
                ty,
            ); // it should
               // be
               // safe to call
               // unwrap here
        }
        self
    }

    /// Generates argument configurations for service methods.
    fn get_arg(&self, input_ty: &str) -> Option<(String, Arg)> {
        match input_ty {
            "google.protobuf.Empty" | "" => None,
            any => {
                let key = convert_ty(any).to_case(Case::Camel);
                let val = Arg {
                    type_of: self.get(any).unwrap_or(any.to_string()),
                    list: false,
                    required: true,
                    /* Setting it not null by default. There's no way to infer this
                     * from proto file */
                    doc: None,
                    modify: None,
                    default_value: None,
                };

                Some((key, val))
            }
        }
    }

    /// Processes proto service definitions and their methods.
    fn append_query_service(mut self, services: Vec<ServiceDescriptorProto>) -> Self {
        let query = self.query.clone();
        if services.is_empty() {
            return self;
        }

        let package = self.package.clone();
        let mut grpc_method = GrpcMethod { package, service: "".to_string(), name: "".to_string() };
        let mut ty = self.config.types.get(&query).cloned().unwrap_or_default();

        for service in services {
            let service_name = service.name().to_string();
            for method in &service.method {
                let method_name = method.name();

                self = self.insert(method_name, DescriptorType::Operation);

                let mut cfg_field = Field::default();
                let arg = self.get_arg(method.input_type());

                if let Some((k, v)) = arg {
                    cfg_field.args.insert(k, v);
                }

                let (output_ty, required) = get_output_ty(method.output_type());
                cfg_field.type_of = self.get(&output_ty).unwrap_or(output_ty.clone());
                cfg_field.required = required;

                grpc_method.service = service_name.clone();
                grpc_method.name = method_name.to_string();

                cfg_field.grpc = Some(Grpc {
                    base_url: None,
                    body: None,
                    group_by: vec![],
                    headers: vec![],
                    method: grpc_method.to_string(),
                });
                ty.fields.insert(
                    self.get(method_name).unwrap_or_else(|| {
                        panic!("Expected key not found in types map: {}", method_name)
                        // it should be safe to call unwrap here
                    }),
                    cfg_field,
                );
            }
        }

        if ty.ne(&Type::default()) {
            self.config.schema.query = Some(query.to_owned());
            self.config.types.insert(query.to_owned(), ty);
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
        x => x,
    }
    .to_string()
}

/// Determines the output type for a service method.
fn get_output_ty(output_ty: &str) -> (String, bool) {
    // type, required
    match output_ty {
        "google.protobuf.Empty" => {
            // If it's no response is expected, we return a nullable string type
            ("String".to_string(), false)
        }
        any => {
            // Setting it not null by default. There's no way to infer this from proto file
            (any.to_string(), true)
        }
    }
}

/// The main entry point that builds a Config object from proto descriptor sets.
pub fn from_proto(descriptor_sets: Vec<FileDescriptorSet>, query: &str) -> Config {
    let mut ctx = Context::new(query);

    for descriptor_set in descriptor_sets {
        for file_descriptor in descriptor_set.file {
            ctx.package = file_descriptor.package().to_string();

            ctx = ctx.append_enums(file_descriptor.enum_type);
            ctx = ctx.append_msg_type(file_descriptor.message_type);
            ctx = ctx.append_query_service(file_descriptor.service.clone());
        }
    }

    ctx.config
}

#[cfg(test)]
mod test {

    use std::path::PathBuf;

    use prost_reflect::prost_types::{FileDescriptorProto, FileDescriptorSet};

    use crate::config::Config;
    use crate::config_generator::from_proto::{from_proto, Context, DescriptorType};

    fn get_proto_file_descriptor(name: &str) -> anyhow::Result<FileDescriptorProto> {
        let mut proto_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        proto_path.push("src");
        proto_path.push("config_generator");
        proto_path.push("proto");
        proto_path.push(name);
        Ok(protox_parse::parse(
            name,
            std::fs::read_to_string(proto_path)?.as_str(),
        )?)
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

        let mut set = FileDescriptorSet::default();

        let news = get_proto_file_descriptor("news.proto")?;
        let greetings_a = get_proto_file_descriptor("greetings_a.proto")?;
        let greetings_b = get_proto_file_descriptor("greetings_b.proto")?;

        set.file.push(news);
        set.file.push(greetings_a);
        set.file.push(greetings_b);

        let result = from_proto(vec![set], "Query").to_sdl();

        insta::assert_snapshot!(result);

        Ok(())
    }

    #[test]
    fn test_config_from_sdl() -> anyhow::Result<()> {
        let mut set = FileDescriptorSet::default();

        let news = get_proto_file_descriptor("news.proto")?;
        let greetings_a = get_proto_file_descriptor("greetings_a.proto")?;
        let greetings_b = get_proto_file_descriptor("greetings_b.proto")?;

        set.file.push(news.clone());
        set.file.push(greetings_a.clone());
        set.file.push(greetings_b.clone());

        let result = from_proto(vec![set], "Query").to_sdl();

        // test for different sets
        let mut set = FileDescriptorSet::default();
        let mut set1 = FileDescriptorSet::default();
        let mut set2 = FileDescriptorSet::default();
        set.file.push(news);
        set1.file.push(greetings_a);
        set2.file.push(greetings_b);

        let result_sets = from_proto(vec![set, set1, set2], "Query").to_sdl();

        pretty_assertions::assert_eq!(result, result_sets);
        Ok(())
    }

    #[test]
    fn test_required_types() -> anyhow::Result<()> {
        // required fields are deprecated in proto3 (https://protobuf.dev/programming-guides/dos-donts/#add-required)
        // this example uses proto2 to test the same.
        // for proto3 it's guaranteed to have a default value (https://protobuf.dev/programming-guides/proto3/#default)
        // and our implementation (https://github.com/tailcallhq/tailcall/pull/1537) supports default values by default.
        // so we do not need to explicitly mark fields as required.

        let mut set = FileDescriptorSet::default();
        let req_proto = get_proto_file_descriptor("person.proto")?;
        set.file.push(req_proto);

        let cfg = from_proto(vec![set], "Query").to_sdl();
        insta::assert_snapshot!(cfg);

        Ok(())
    }
    #[test]
    fn test_get_value() {
        let mut ctx: Context = Context::new("Query").package("com.example".to_string());
        assert_eq!(
            ctx.get_name("TestEnum", DescriptorType::Enum),
            "ComExample__TestEnum"
        );
        assert_eq!(
            ctx.get_name("testMessage", DescriptorType::Message),
            "ComExample__testMessage"
        );
        assert_eq!(
            ctx.get_name("QueryName", DescriptorType::Operation),
            "ComExample__queryName"
        );
    }

    #[test]
    fn test_insert_and_get() {
        let mut ctx: Context = Context::new("Query").package("com.example".to_string());
        ctx = ctx.insert("TestEnum", DescriptorType::Enum);
        assert_eq!(
            ctx.get("TestEnum"),
            Some("ComExample__TestEnum".to_string())
        );
        ctx = ctx.insert("testMessage", DescriptorType::Message);
        assert_eq!(
            ctx.get("testMessage"),
            Some("ComExample__testMessage".to_string())
        );
        assert_eq!(ctx.get("NonExisting"), None);
    }
}
