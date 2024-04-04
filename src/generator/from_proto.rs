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
pub(super) static DEFAULT_PACKAGE_SEPARATOR: &str = "_";

/// Enum to represent the type of the descriptor
#[derive(Display, Clone)]
enum DescriptorType {
    Enum,
    Message,
    Operation,
}

impl DescriptorType {
    fn as_str_name(&self, package: &str, name: &str) -> String {
        let package = package.replace('.', DEFAULT_PACKAGE_SEPARATOR);
        if package.is_empty() {
            return match self {
                DescriptorType::Operation => name.to_case(Case::Camel).to_string(),
                _ => name.to_string(),
            };
        }
        match self {
            DescriptorType::Enum => {
                format!(
                    "{}{}{}",
                    package.to_case(Case::UpperCamel),
                    DEFAULT_SEPARATOR,
                    name
                )
            }
            DescriptorType::Message => {
                format!(
                    "{}{}{}",
                    package.to_case(Case::UpperCamel),
                    DEFAULT_SEPARATOR,
                    name
                )
            }
            DescriptorType::Operation => name.to_case(Case::Camel).to_string(),
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
        ty.as_str_name(&self.package, name)
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

    /// Resolves the actual name and inserts the type.
    fn insert_type(mut self, name: &str, ty: Type) -> Self {
        if let Some(name) = self.get(name) {
            self.config.types.insert(name, ty);
        }
        self
    }

    /// Retrieves or creates a Type configuration for a given proto type.
    fn get_ty(&self, name: &str) -> Type {
        let mut ty = self
            .get(name)
            .and_then(|name| self.config.types.get(&name))
            .cloned()
            .unwrap_or_default();

        let id = if self.package.is_empty() {
            name.to_string()
        } else {
            format!("{}.{}", self.package, name)
        };

        ty.tag = Some(Tag { id });
        ty
    }

    /// Processes proto enum types.
    fn append_enums(mut self, enums: &Vec<EnumDescriptorProto>) -> Self {
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

            self = self.insert(&msg_name, DescriptorType::Message);
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
            self.config = Config::default();
            return self;
        }

        let package = self.package.clone();
        let mut grpc_method = GrpcMethod { package, service: "".to_string(), name: "".to_string() };

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

                let output_ty = get_output_ty(method.output_type());
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

    ctx.config
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use prost_reflect::prost_types::{FileDescriptorProto, FileDescriptorSet};

    use crate::generator::from_proto::{from_proto, Context, DescriptorType};

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

        let actual = ctx.get_name("TestEnum", DescriptorType::Enum);
        let expected = "ComExample__TestEnum";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_get_value_message() {
        let ctx: Context = Context::new("Query").package("com.example".to_string());

        let actual = ctx.get_name("testMessage", DescriptorType::Message);
        let expected = "ComExample__testMessage";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_get_value_query_name() {
        let ctx: Context = Context::new("Query").package("com.example".to_string());

        let actual = ctx.get_name("QueryName", DescriptorType::Operation);
        let expected = "queryName";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_insert_and_get_enum() {
        let ctx: Context = Context::new("Query")
            .package("com.example".to_string())
            .insert("TestEnum", DescriptorType::Enum);

        let actual = ctx.get("TestEnum");
        let expected = Some("ComExample__TestEnum".to_string());
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_insert_and_get_message() {
        let ctx: Context = Context::new("Query")
            .package("com.example".to_string())
            .insert("testMessage", DescriptorType::Message);
        let actual = ctx.get("testMessage");
        let expected = Some("ComExample__testMessage".to_string());
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
