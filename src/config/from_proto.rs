#![allow(dead_code)] // TODO check what to do..

use std::collections::BTreeMap;

use convert_case::{Case, Casing};
use prost_reflect::prost_types::{
    DescriptorProto, EnumDescriptorProto, FileDescriptorSet, ServiceDescriptorProto,
};

use crate::blueprint::GrpcMethod;
use crate::config::{Arg, Config, Field, Grpc, ProtoGeneratorConfig, Type};

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

fn append_enums(map: &mut Config, enums: Vec<EnumDescriptorProto>) {
    for enum_ in enums {
        let ty = Type {
            variants: Some(enum_.value.iter().map(|v| v.name().to_string()).collect()),
            ..Default::default()
        };
        map.types.insert(enum_.name().to_string(), ty);
    }
}

fn append_msg_type(config: &mut Config, messages: Vec<DescriptorProto>) {
    if messages.is_empty() {
        return;
    }
    for message in messages {
        let msg_name = message.name().to_string();
        append_enums(config, message.enum_type);
        append_msg_type(config, message.nested_type);

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
        config.types.insert(msg_name, ty);
    }
}

fn generate_ty(
    map: &mut BTreeMap<String, Type>,
    services: &[ServiceDescriptorProto],
    package: String,
    key: &str,
) -> Type {
    let mut grpc_method = GrpcMethod { package, service: "".to_string(), name: "".to_string() };
    let mut ty = map.get(key).cloned().unwrap_or_default();

    for service in services {
        let service_name = service.name().to_string();
        for method in &service.method {
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
    ty
}

fn append_query_service(
    config: &mut Config,
    services: &[ServiceDescriptorProto],
    gen: &ProtoGeneratorConfig,
    package: String,
) {
    let query = gen.query();

    if services.is_empty()
        || !services
            .iter()
            .any(|x| !gen.is_mutation(x.name().to_string()))
    {
        return;
    } else {
        config.schema.query = Some(query.to_string());
    }
    let ty = generate_ty(&mut config.types, services, package, query);
    config.types.insert(query.to_string(), ty);
}

fn append_mutation_service(
    config: &mut Config,
    services: &[ServiceDescriptorProto],
    gen: &ProtoGeneratorConfig,
    package: String,
) {
    let mutation = gen.mutation();
    if services.is_empty()
        || !services
            .iter()
            .any(|x| gen.is_mutation(x.name().to_string()))
    {
        return;
    } else {
        config.schema.mutation = Some(mutation.to_string());
    }
    let ty = generate_ty(&mut config.types, services, package, mutation);
    config.types.insert(mutation.to_string(), ty);
}

pub fn from_proto(descriptor_sets: Vec<FileDescriptorSet>, gen: ProtoGeneratorConfig) -> Config {
    let mut config = Config::default();

    for descriptor_set in descriptor_sets {
        for file_descriptor in descriptor_set.file {
            let pkg_name = file_descriptor.package().to_string();

            append_enums(&mut config, file_descriptor.enum_type);
            append_msg_type(&mut config, file_descriptor.message_type);
            append_query_service(
                &mut config,
                &file_descriptor.service,
                &gen,
                pkg_name.clone(),
            );
            append_mutation_service(&mut config, &file_descriptor.service, &gen, pkg_name);
        }
    }

    config
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use prost_reflect::prost_types::FileDescriptorSet;

    use crate::config::from_proto::from_proto;
    use crate::config::ProtoGeneratorConfig;

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

        set.file.push(news.clone());
        set.file.push(greetings.clone());

        let result = from_proto(vec![set], ProtoGeneratorConfig::default()).to_sdl();

        insta::assert_snapshot!(result);

        // test for 2 different sets
        let mut set = FileDescriptorSet::default();
        let mut set1 = FileDescriptorSet::default();
        set.file.push(news);
        set1.file.push(greetings);

        let result_sets = from_proto(vec![set, set1], ProtoGeneratorConfig::default()).to_sdl();

        assert_eq!(result, result_sets);

        Ok(())
    }
}
