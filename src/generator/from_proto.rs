use std::collections::{BTreeSet, HashMap};

use convert_case::{Case, Casing};
use derive_setters::Setters;
use prost_reflect::prost_types::{
    DescriptorProto, EnumDescriptorProto, FileDescriptorSet, ServiceDescriptorProto,
};

use crate::blueprint::GrpcMethod;
use crate::config::{Arg, Config, Field, Grpc, Tag, Type};

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

    /// Processes proto enum types.
    fn append_enums(mut self, enums: &Vec<EnumDescriptorProto>) -> Self {
        for enum_ in enums {
            let enum_name = enum_.name();

            self = self.insert(enum_name, NameConvertor::Enum);
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
            self = self.insert_type(enum_name, ty);
        }
        self
    }

    /// Processes proto message types.
    fn append_msg_type(mut self, messages: &Vec<DescriptorProto>) -> Self {
        if messages.is_empty() {
            return self;
        }
        for message in messages {
            let msg_name = message.name().to_string();
            if let Some(options) = message.options.as_ref() {
                if options.map_entry.unwrap_or_default() {
                    continue;
                }
            }

            self = self.insert(&msg_name, NameConvertor::Message);
            let mut ty = self.get_ty(&msg_name);

            self = self.append_enums(&message.enum_type);
            self = self.append_msg_type(&message.nested_type);

            for field in message.field.iter() {
                let field_name = field.name().to_string();
                let mut cfg_field = Field::default();

                let label = field.label().as_str_name().to_lowercase();
                cfg_field.list = label.contains("repeated");
                cfg_field.required = label.contains("required");

                if field.r#type.is_some() {
                    let type_of = convert_ty(field.r#type().as_str_name());
                    if type_of.eq("JSON") {
                        cfg_field.list = false;
                    }
                    cfg_field.type_of = type_of;
                } else {
                    // for non-primitive types
                    let type_of = convert_ty(field.type_name());
                    cfg_field.type_of = self.get(&type_of).unwrap_or(type_of);
                }

                ty.fields.insert(field_name.to_case(Case::Camel), cfg_field);
            }

            self = self.insert_type(&msg_name, ty);
        }
        self
    }

    fn append_nested_package(mut self, method_name: String, field: Field) -> Self {
        let split = self
            .package
            .split('.')
            .collect::<Vec<&str>>()
            .iter()
            .filter(|x| !x.is_empty())
            .map(|x| x.to_case(Case::UpperCamel))
            .collect::<Vec<String>>();
        // let n = len(split)
        // len(types) = n
        // len(fields) = n-1
        let n = split.len();

        for (i, type_name) in split.iter().enumerate() {
            if i == 0 {
                let mut ty = self
                    .config
                    .types
                    .get(&self.query)
                    .cloned()
                    .unwrap_or_default();
                let field = Field::default().type_of(type_name.clone());
                ty.fields.insert(type_name.to_case(Case::Camel), field);
                self.config.schema.query = Some(self.query.to_owned());
                self.config.types.insert(self.query.to_owned(), ty);
            }
            if i + 1 < n {
                let field_name = &split[i + 1];
                let field = Field::default().type_of(field_name.clone());
                let mut ty = Type::default();
                ty.fields.insert(field_name.to_case(Case::Camel), field);
                self.config.types.insert(type_name.clone(), ty);
            } else if let Some(ty) = self.config.types.get_mut(type_name) {
                ty.fields.insert(method_name.clone(), field.clone());
            } else {
                let mut ty = Type::default();
                ty.fields.insert(method_name.clone(), field.clone());
                self.config.types.insert(type_name.clone(), ty);
            }
        }

        if n == 0 {
            let mut ty = self
                .config
                .types
                .get(&self.query)
                .cloned()
                .unwrap_or_default();
            ty.fields.insert(method_name.to_case(Case::Camel), field);
            self.config.schema.query = Some(self.query.to_owned());
            self.config.types.insert(self.query.to_owned(), ty);
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
                let method_name = method.name();

                self = self.insert(method_name, NameConvertor::Method);

                let mut cfg_field = Field::default();
                if let Some(arg_type) = get_input_ty(method.input_type()) {
                    let key = self.get_name(&convert_ty(&arg_type), NameConvertor::Arg);
                    let type_of = self.get_name(&arg_type, NameConvertor::Message);
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
                self = self.insert(&output_ty, NameConvertor::Message);
                cfg_field.type_of = self.get(&output_ty).unwrap_or(output_ty.clone());
                cfg_field.required = true;

                grpc_method.service = service_name.clone();
                grpc_method.name = method_name.to_string();
                let grpc_method_string = grpc_method.to_string();

                let method = if let Some(stripped) = grpc_method_string.strip_prefix('.') {
                    stripped.to_string()
                } else {
                    grpc_method_string
                };

                cfg_field.grpc = Some(Grpc {
                    base_url: None,
                    body: None,
                    group_by: vec![],
                    headers: vec![],
                    method,
                });

                if let Some(method_name) = self.get(method_name) {
                    self = self.append_nested_package(method_name, cfg_field);
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

    ctx.config.remove_unused_types();

    ctx.config
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use prost_reflect::prost_types::{FileDescriptorProto, FileDescriptorSet};

    use crate::generator::from_proto::{from_proto, Context, NameConvertor};

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
        let set = new_file_desc(&["greetings.proto", "greetings_message.proto"]).unwrap();
        let result = from_proto(&[set], "Query").to_sdl();
        insta::assert_snapshot!(dbg!(result));
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

    #[test]
    fn test_get_value_enum() {
        let ctx: Context = Context::new("Query").package("com.example".to_string());

        let actual = ctx.get_name("TestEnum", NameConvertor::Enum);
        let expected = "Com_Example__TestEnum";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_get_value_message() {
        let ctx: Context = Context::new("Query").package("com.example".to_string());

        let actual = ctx.get_name("testMessage", NameConvertor::Message);
        let expected = "Com_Example__TestMessage";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_get_value_query_name() {
        let ctx: Context = Context::new("Query").package("com.example".to_string());

        let actual = ctx.get_name("QueryName", NameConvertor::Method);
        let expected = "queryName";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_insert_and_get_enum() {
        let ctx: Context = Context::new("Query")
            .package("com.example".to_string())
            .insert("TestEnum", NameConvertor::Enum);

        let actual = ctx.get("TestEnum");
        let expected = Some("Com_Example__TestEnum".to_string());
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_insert_and_get_message() {
        let ctx: Context = Context::new("Query")
            .package("com.example".to_string())
            .insert("testMessage", NameConvertor::Message);
        let actual = ctx.get("testMessage");
        let expected = Some("Com_Example__TestMessage".to_string());
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_insert_and_get_non_existing() {
        let ctx: Context = Context::new("Query").package("com.example".to_string());
        let actual = ctx.get("NonExisting");
        let expected = None;
        assert_eq!(actual, expected);
    }
}
