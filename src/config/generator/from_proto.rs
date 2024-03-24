use std::collections::{BTreeSet, HashMap};

use anyhow::anyhow;
use convert_case::{Case, Casing};
use prost_reflect::prost_types::{
    DescriptorProto, EnumDescriptorProto, FileDescriptorSet, ServiceDescriptorProto,
};

use crate::blueprint::GrpcMethod;
use crate::config::generator::proto_generator::{
    ConfigWrapper, DescriptorType, Options, ProtoGeneratorConfig, DEFAULT_SPECTATOR,
};
use crate::config::{Arg, Field, Grpc, Type};

struct Helper {
    map: HashMap<String, String>,
    package: String,
    options: Options,
}

impl Helper {
    fn with_options(options: Options) -> Self {
        Self { map: Default::default(), package: "".to_string(), options }
    }

    fn get_value(&self, name: &str, ty: DescriptorType) -> String {
        match self.options {
            Options::AppendPkgId => match ty {
                DescriptorType::Enum => {
                    format!("{}{}{}", name, DEFAULT_SPECTATOR, self.package)
                }
                DescriptorType::Message => {
                    format!("{}{}{}", name, DEFAULT_SPECTATOR, self.package)
                }
                DescriptorType::Method => format!(
                    "{}{}{}",
                    name.to_case(Case::Camel),
                    DEFAULT_SPECTATOR,
                    self.package
                ),
            },
            _ => match ty {
                DescriptorType::Enum => name.to_string(),
                DescriptorType::Message => name.to_string(),
                DescriptorType::Method => name.to_case(Case::Camel),
            },
        }
    }
    fn insert(&mut self, name: &str, ty: DescriptorType) -> anyhow::Result<()> {
        if Options::FailIfCollide == self.options {
            if self
                .map
                .keys()
                .any(|v| v.split('.').last().map(|c| c.eq(name)).unwrap_or_default())
            {
                return Err(anyhow!("Duplicate keys found for: {}", name));
            } else {
                self.map.insert(name.to_string(), self.get_value(name, ty));
            }
        } else if Options::Merge == self.options {
            self.map.insert(name.to_string(), self.get_value(name, ty));
        } else {
            self.map.insert(
                format!("{}.{}", self.package, name),
                self.get_value(name, ty),
            );
        }
        Ok(())
    }
    fn get(&self, name: &str) -> Option<String> {
        match self.options {
            Options::AppendPkgId => self.map.get(&format!("{}.{}", self.package, name)).cloned(),
            _ => self.map.get(name).cloned(),
        }
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

fn append_enums(
    config_wrapper: &mut ConfigWrapper,
    enums: Vec<EnumDescriptorProto>,
    helper: &mut Helper,
) -> anyhow::Result<()> {
    for enum_ in enums {
        let enum_name = enum_.name();

        let enum_ty = DescriptorType::Enum;
        helper.insert(enum_name, enum_ty)?;

        let mut ty = config_wrapper.get_ty(&helper.get(enum_name).unwrap());

        let mut variants = enum_
            .value
            .iter()
            .map(|v| v.name().to_string())
            .collect::<BTreeSet<String>>();
        if let Some(vars) = ty.variants {
            variants.extend(vars);
        }
        ty.variants = Some(variants);

        config_wrapper.insert_ty(helper.get(enum_name).unwrap(), ty, enum_ty.to_string());
        // it should be
        // safe to call
        // unwrap here
    }
    Ok(())
}

fn append_msg_type(
    config_wrapper: &mut ConfigWrapper,
    messages: Vec<DescriptorProto>,
    helper: &mut Helper,
) -> anyhow::Result<()> {
    if messages.is_empty() {
        return Ok(());
    }
    for message in messages {
        let msg_name = message.name().to_string();

        let msg_ty = DescriptorType::Message;
        helper.insert(&msg_name, msg_ty)?;

        append_enums(config_wrapper, message.enum_type, helper)?;
        append_msg_type(config_wrapper, message.nested_type, helper)?;

        let mut ty = config_wrapper.get_ty(&helper.get(&msg_name).unwrap()); // it should be
                                                                             // safe to call
                                                                             // unwrap here

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

        config_wrapper.insert_ty(helper.get(&msg_name).unwrap(), ty, msg_ty.to_string());
        // it should be
        // safe to call
        // unwrap here
    }
    Ok(())
}

fn generate_ty(
    config_wrapper: &mut ConfigWrapper,
    services: Vec<ServiceDescriptorProto>,
    helper: &mut Helper,
    key: &str,
) -> anyhow::Result<Type> {
    let package = helper.package.clone();
    let mut grpc_method = GrpcMethod { package, service: "".to_string(), name: "".to_string() };
    let mut ty = config_wrapper.get_ty(key);

    for service in services {
        let service_name = service.name().to_string();
        for method in &service.method {
            let method_name = method.name();

            helper.insert(method_name, DescriptorType::Method)?;

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
    config_wrapper: &mut ConfigWrapper,
    mut services: Vec<ServiceDescriptorProto>,
    gen: &ProtoGeneratorConfig,
    helper: &mut Helper,
) -> anyhow::Result<()> {
    let query = gen.get_query();

    if services.is_empty() {
        return Ok(());
    }

    for service in services.iter_mut() {
        service.method.retain(|v| !gen.is_mutation(v.name()));
    }

    let ty = generate_ty(config_wrapper, services, helper, query)?;

    if ty.ne(&Type::default()) {
        config_wrapper.config.schema.query = Some(query.to_string());
        config_wrapper.insert_ty(query.to_string(), ty, query.to_string());
    }
    Ok(())
}

fn append_mutation_service(
    config_wrapper: &mut ConfigWrapper,
    mut services: Vec<ServiceDescriptorProto>,
    gen: &ProtoGeneratorConfig,
    helper: &mut Helper,
) -> anyhow::Result<()> {
    let mutation = gen.get_mutation();
    if services.is_empty() {
        return Ok(());
    }

    for service in services.iter_mut() {
        service.method.retain(|v| gen.is_mutation(v.name()));
    }

    let ty = generate_ty(config_wrapper, services, helper, mutation)?;
    if ty.ne(&Type::default()) {
        config_wrapper.config.schema.mutation = Some(mutation.to_string());
        config_wrapper.insert_ty(mutation.to_string(), ty, mutation.to_string());
    }
    Ok(())
}

pub fn prebuild_config(
    descriptor_sets: Vec<FileDescriptorSet>,
    gen: &ProtoGeneratorConfig,
    options: Options,
) -> anyhow::Result<ConfigWrapper> {
    let mut config = ConfigWrapper::default();
    let mut helper = Helper::with_options(options);

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
            append_mutation_service(&mut config, file_descriptor.service, gen, &mut helper)?;
        }
    }

    Ok(config)
}

#[cfg(test)]
mod test {

    use std::path::PathBuf;

    use prost_reflect::prost_types::{FileDescriptorProto, FileDescriptorSet};

    use crate::config::generator::from_proto::prebuild_config;
    use crate::config::generator::proto_generator::{
        Options, ProtoGeneratorConfig, ProtoGeneratorFxn,
    };

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
        let is_mut = |x: &str| !x.starts_with("Get");
        ProtoGeneratorConfig::new(
            Some("Query".to_string()),
            Some("Mutation".to_string()),
            ProtoGeneratorFxn { is_mutation: Box::new(is_mut), ..Default::default() },
        )
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

        let result =
            prebuild_config(vec![set], &get_generator_cfg(), Options::AppendPkgId)?.to_sdl();

        insta::assert_snapshot!(result);

        // test for 2 different sets
        let mut set = FileDescriptorSet::default();
        let mut set1 = FileDescriptorSet::default();
        let mut set2 = FileDescriptorSet::default();
        set.file.push(news);
        set1.file.push(greetings);
        set2.file.push(greetings_dup_methods);

        let result_sets = prebuild_config(
            vec![set, set1, set2],
            &get_generator_cfg(),
            Options::AppendPkgId,
        )?
        .to_sdl();

        assert_eq!(result, result_sets);

        Ok(())
    }

    #[test]
    fn test_merge() -> anyhow::Result<()> {
        let mut set = FileDescriptorSet::default();

        let news = get_proto_file_descriptor("news_enum.proto")?;
        let greetings = get_proto_file_descriptor("greetings.proto")?;
        let greetings_dup_methods = get_proto_file_descriptor("greetings_dup_methods.proto")?;

        set.file.push(news.clone());
        set.file.push(greetings.clone());
        set.file.push(greetings_dup_methods.clone());

        let result = prebuild_config(vec![set], &get_generator_cfg(), Options::Merge)?.to_sdl();

        insta::assert_snapshot!(result);
        Ok(())
    }

    #[test]
    fn test_fail_if_collide() -> anyhow::Result<()> {
        let mut set = FileDescriptorSet::default();

        let news = get_proto_file_descriptor("news_enum.proto")?;
        let greetings = get_proto_file_descriptor("greetings.proto")?;
        let greetings_dup_methods = get_proto_file_descriptor("greetings_dup_methods.proto")?;

        set.file.push(news.clone());
        set.file.push(greetings.clone());
        set.file.push(greetings_dup_methods.clone());

        let result = prebuild_config(vec![set], &get_generator_cfg(), Options::FailIfCollide);
        assert!(result.is_err());
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

        let cfg =
            prebuild_config(vec![set], &get_generator_cfg(), Options::FailIfCollide)?.to_sdl();
        insta::assert_snapshot!(cfg);

        Ok(())
    }
}
