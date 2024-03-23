#![allow(dead_code)] // TODO check what to do..

use std::collections::BTreeMap;

use convert_case::{Case, Casing};
use derive_setters::Setters;
use prost_reflect::prost_types::{
    DescriptorProto, EnumDescriptorProto, FileDescriptorSet, ServiceDescriptorProto,
};

use crate::blueprint::GrpcMethod;
use crate::config::{Arg, Config, Field, Grpc, Type};

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

fn get_arg(input_ty: &str) -> Option<(String, Arg)> {
    match input_ty {
        "google.protobuf.Empty" | "" => None,
        any => {
            let key = convert_ty(any).to_case(Case::Camel);
            let val = Arg {
                type_of: any.to_string(),
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

fn append_enums(map: &mut BTreeMap<String, Type>, enums: Vec<EnumDescriptorProto>) {
    for enum_ in enums {
        let ty = Type {
            variants: Some(enum_.value.iter().map(|v| v.name().to_string()).collect()),
            ..Default::default()
        };
        map.insert(enum_.name().to_string(), ty);
    }
}

fn append_msg_type(map: &mut BTreeMap<String, Type>, messages: Vec<DescriptorProto>) {
    if messages.is_empty() {
        return;
    }
    for message in messages {
        let msg_name = message.name().to_string();
        append_enums(map, message.enum_type);
        append_msg_type(map, message.nested_type);

        let mut ty = Type::default();

        for field in message.field {
            let field_name = field.name().to_string();
            let mut cfg_field = Field::default();

            let label = field.label().as_str_name().to_lowercase();
            cfg_field.list = label.contains("repeated");
            cfg_field.required = label.contains("required");

            if field.r#type.is_some() {
                // for non-primitive types
                let type_of = convert_ty(field.r#type().as_str_name());
                cfg_field.type_of = type_of.to_string();
            } else {
                cfg_field.type_of = convert_ty(field.type_name());
            }

            ty.fields.insert(field_name, cfg_field);
        }
        map.insert(msg_name, ty);
    }
}

fn append_service(
    map: &mut BTreeMap<String, Type>,
    services: Vec<ServiceDescriptorProto>,
    query: String,
    package: String,
) {
    if services.is_empty() {
        return;
    }
    let mut grpc_method = GrpcMethod { package, service: "".to_string(), name: "".to_string() };
    let mut ty = map.get(&query).cloned().unwrap_or_default();

    for service in services {
        let service_name = service.name().to_string();
        for method in service.method {
            let mut cfg_field = Field::default();
            if let Some((k, v)) = get_arg(method.input_type()) {
                cfg_field.args.insert(k, v);
            }

            let (output_ty, required) = get_output_ty(method.output_type());
            cfg_field.type_of = output_ty;
            cfg_field.required = required;

            grpc_method.service = service_name.clone();
            grpc_method.name = method.name().to_string();

            cfg_field.grpc = Some(Grpc {
                base_url: None,
                body: None,
                group_by: vec![],
                headers: vec![],
                method: grpc_method.to_string(),
            });
            ty.fields
                .insert(method.name().to_case(Case::Camel), cfg_field);
        }
    }
    map.insert(query, ty);
}

pub fn from_proto(descriptor_sets: Vec<FileDescriptorSet>, gen: ProtoGenerator) -> Config {
    let mut config = Config::default();
    let mut types = BTreeMap::new();
    let query = gen.query;
    config.schema.query = Some(query.clone());

    for descriptor_set in descriptor_sets {
        for file_descriptor in descriptor_set.file {
            let pkg_name = file_descriptor.package().to_string();

            append_enums(&mut types, file_descriptor.enum_type);
            append_msg_type(&mut types, file_descriptor.message_type);
            append_service(&mut types, file_descriptor.service, query.clone(), pkg_name);
        }
    }

    config.types = types;

    config
}

// FIXME: @ssddOnTop move it to it's own file.
#[derive(Setters)]
pub struct ProtoGenerator {
    query: String,
    mutation: String,
    is_mutation: Box<dyn Fn(String) -> bool>,
}

impl Default for ProtoGenerator {
    fn default() -> Self {
        Self {
            query: Default::default(),
            mutation: Default::default(),
            is_mutation: Box::new(|_| false),
        }
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use prost_reflect::prost_types::FileDescriptorSet;

    use crate::config::from_proto::{from_proto, ProtoGenerator};

    #[test]
    fn test_from_proto() -> anyhow::Result<()> {
        let mut set = FileDescriptorSet::default();
        let mut proto_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        proto_path.push("src");
        proto_path.push("grpc");
        proto_path.push("tests");
        proto_path.push("proto");

        let mut news_enum = proto_path.clone();
        news_enum.push("news_enum.proto");

        let mut greetings = proto_path;
        greetings.push("greetings.proto");

        let news = protox_parse::parse("news.proto", std::fs::read_to_string(news_enum)?.as_str())?;

        let greetings = protox_parse::parse(
            "greetings.proto",
            std::fs::read_to_string(greetings)?.as_str(),
        )?;

        set.file.push(news);
        set.file.push(greetings);

        let result = from_proto(vec![set], ProtoGenerator::default()).to_sdl();

        insta::assert_snapshot!(result);
        Ok(())
    }
}
