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

pub(super) static DEFAULT_SPECTATOR: &str = "_";

/// Contains the configuration for the config generator
pub struct ProtoGeneratorConfig {
    query: String,
}

impl ProtoGeneratorConfig {
    pub fn new(query: Option<String>) -> Self {
        Self { query: query.unwrap_or_else(|| "Query".to_string()) }
    }

    pub fn get_query(&self) -> &str {
        self.query.as_str()
    }
}

/// Enum to represent the type of the descriptor
#[derive(Display, Clone)]
pub enum DescriptorType {
    Enum,
    Message,
    Query,
}

#[derive(Default)]
struct Helper {
    map: HashMap<String, String>,
    package: String,
}

impl Helper {
    fn get_value(&self, name: &str, ty: DescriptorType) -> String {
        let package = self.package.replace('.', DEFAULT_SPECTATOR).to_uppercase();
        match ty {
            DescriptorType::Enum => {
                format!("{}{}{}", package, DEFAULT_SPECTATOR, name)
            }
            DescriptorType::Message => {
                format!("{}{}{}", package, DEFAULT_SPECTATOR, name)
            }
            DescriptorType::Query => format!(
                "{}{}{}",
                package.to_case(Case::Snake),
                DEFAULT_SPECTATOR,
                name.to_case(Case::Camel),
            ),
        }
    }
    fn insert(&mut self, name: &str, ty: DescriptorType) {
        self.map.insert(
            format!("{}.{}", self.package, name),
            self.get_value(name, ty),
        );
    }
    fn get(&self, name: &str) -> Option<String> {
        self.map.get(&format!("{}.{}", self.package, name)).cloned()
    }
}

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

fn get_output_ty(output_ty: &str) -> (String, bool) {
    // type, required
    match output_ty {
        "google.protobuf.Empty" => {
            ("String".to_string(), false) // If it's no response is expected, we
                                          // return a nullable string type
        }
        any => (any.to_string(), true), /* Setting it not null by default. There's no way to
                                         * infer this from proto file */
    }
}

fn get_arg(input_ty: &str, helper: &mut Helper) -> Option<(String, Arg)> {
    match input_ty {
        "google.protobuf.Empty" | "" => None,
        any => {
            let key = convert_ty(any).to_case(Case::Camel);
            let val = Arg {
                type_of: helper.get(any).unwrap_or(any.to_string()),
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

fn get_ty(name: &str, cfg: &Config, helper: &mut Helper, ty: DescriptorType) -> Type {
    helper.insert(name, ty);
    let mut ty = cfg
        .types
        .get(&helper.get(name).unwrap())
        .cloned()
        .unwrap_or_default(); // it should be
                              // safe to call
                              // unwrap here
    ty.tag = Some(Tag { name: format!("{}.{}", helper.package, name) });
    ty
}

fn append_enums(
    config: &mut Config,
    enums: Vec<EnumDescriptorProto>,
    helper: &mut Helper,
) -> anyhow::Result<()> {
    for enum_ in enums {
        let enum_name = enum_.name();

        let mut ty = get_ty(enum_name, config, helper, DescriptorType::Enum);

        let mut variants = enum_
            .value
            .iter()
            .map(|v| v.name().to_string())
            .collect::<BTreeSet<String>>();
        if let Some(vars) = ty.variants {
            variants.extend(vars);
        }
        ty.variants = Some(variants);
        config.types.insert(helper.get(enum_name).unwrap(), ty);
        // it should be
        // safe to call
        // unwrap here
    }
    Ok(())
}

fn append_msg_type(
    config: &mut Config,
    messages: Vec<DescriptorProto>,
    helper: &mut Helper,
) -> anyhow::Result<()> {
    if messages.is_empty() {
        return Ok(());
    }
    for message in messages {
        let msg_name = message.name().to_string();

        let mut ty = get_ty(&msg_name, config, helper, DescriptorType::Message);

        append_enums(config, message.enum_type, helper)?;
        append_msg_type(config, message.nested_type, helper)?;

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
                cfg_field.type_of = helper.get(&type_of).unwrap_or(type_of);
            }

            ty.fields.insert(field_name, cfg_field);
        }

        config.types.insert(helper.get(&msg_name).unwrap(), ty); // it should be
                                                                 // safe to call
                                                                 // unwrap here
    }
    Ok(())
}

fn generate_ty(
    config: &mut Config,
    services: Vec<ServiceDescriptorProto>,
    helper: &mut Helper,
    key: &str,
) -> anyhow::Result<Type> {
    let package = helper.package.clone();
    let mut grpc_method = GrpcMethod { package, service: "".to_string(), name: "".to_string() };
    let mut ty = config.types.get(key).cloned().unwrap_or_default();

    for service in services {
        let service_name = service.name().to_string();
        for method in &service.method {
            let method_name = method.name();

            helper.insert(method_name, DescriptorType::Query);

            let mut cfg_field = Field::default();
            if let Some((k, v)) = get_arg(method.input_type(), helper) {
                cfg_field.args.insert(k, v);
            }

            let (output_ty, required) = get_output_ty(method.output_type());
            cfg_field.type_of = helper.get(&output_ty).unwrap_or(output_ty.clone());
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
            ty.fields
                .insert(helper.get(method_name).unwrap(), cfg_field);
        }
    }
    Ok(ty)
}

fn append_query_service(
    config: &mut Config,
    services: Vec<ServiceDescriptorProto>,
    gen: &ProtoGeneratorConfig,
    helper: &mut Helper,
) -> anyhow::Result<()> {
    if services.is_empty() {
        return Ok(());
    }

    let query = gen.get_query();

    let ty = generate_ty(config, services, helper, query)?;

    if ty.ne(&Type::default()) {
        config.schema.query = Some(query.to_string());
        config.types.insert(query.to_string(), ty);
    }
    Ok(())
}

pub fn build_config(
    descriptor_sets: Vec<FileDescriptorSet>,
    gen: &ProtoGeneratorConfig,
) -> anyhow::Result<Config> {
    let mut config = Config::default();
    let mut helper = Helper::default();

    for descriptor_set in descriptor_sets {
        for file_descriptor in descriptor_set.file {
            helper.package = file_descriptor.package().to_string();

            append_enums(&mut config, file_descriptor.enum_type, &mut helper)?;
            append_msg_type(&mut config, file_descriptor.message_type, &mut helper)?;
            append_query_service(
                &mut config,
                file_descriptor.service.clone(),
                gen,
                &mut helper,
            )?;
        }
    }

    Ok(config)
}

#[cfg(test)]
mod test {

    use std::path::PathBuf;

    use prost_reflect::prost_types::{FileDescriptorProto, FileDescriptorSet};

    use crate::config_generator::from_proto::{build_config, ProtoGeneratorConfig};

    fn get_proto_file_descriptor(name: &str) -> anyhow::Result<FileDescriptorProto> {
        let mut proto_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        proto_path.push("src");
        proto_path.push("grpc");
        proto_path.push("tests");
        proto_path.push("proto");
        proto_path.push(name);
        Ok(protox_parse::parse(
            name,
            std::fs::read_to_string(proto_path)?.as_str(),
        )?)
    }

    fn get_generator_cfg() -> ProtoGeneratorConfig {
        ProtoGeneratorConfig::new(Some("Query".to_string()))
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

        let news = get_proto_file_descriptor("news_enum.proto")?;
        let greetings = get_proto_file_descriptor("greetings.proto")?;
        let greetings_dup_methods = get_proto_file_descriptor("greetings_dup_methods.proto")?;

        set.file.push(news.clone());
        set.file.push(greetings.clone());
        set.file.push(greetings_dup_methods.clone());

        let result = build_config(vec![set], &get_generator_cfg())?.to_sdl();

        insta::assert_snapshot!(result);

        // test for 2 different sets
        let mut set = FileDescriptorSet::default();
        let mut set1 = FileDescriptorSet::default();
        let mut set2 = FileDescriptorSet::default();
        set.file.push(news);
        set1.file.push(greetings);
        set2.file.push(greetings_dup_methods);

        let result_sets = build_config(vec![set, set1, set2], &get_generator_cfg())?.to_sdl();

        assert_eq!(result, result_sets);

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
        let req_proto = get_proto_file_descriptor("required_fields.proto")?;
        set.file.push(req_proto);

        let cfg = build_config(vec![set], &get_generator_cfg())?.to_sdl();
        insta::assert_snapshot!(cfg);

        Ok(())
    }
}
