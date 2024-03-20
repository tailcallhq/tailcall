#![allow(dead_code)] // TODO check what to do..

use std::collections::BTreeMap;

use convert_case::{Case, Casing};
use prost_reflect::prost_types::{
    DescriptorProto, EnumDescriptorProto, FileDescriptorSet, ServiceDescriptorProto,
};

use crate::config::{Arg, Config, Field, Type};

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
) {
    if services.is_empty() {
        return;
    }
    let mut ty = Type::default();

    for service in services {
        for method in service.method {
            let mut cfg_field = Field::default();
            if let Some((k, v)) = get_arg(method.input_type()) {
                cfg_field.args.insert(k, v);
            }

            let (output_ty, required) = get_output_ty(method.output_type());
            cfg_field.type_of = output_ty;
            cfg_field.required = required;
            let name = method.name().to_case(Case::Camel);
            ty.fields.insert(name, cfg_field);
        }
    }
    map.insert(query, ty);
}

pub fn from_proto(
    descriptor_set: FileDescriptorSet,
    query: Option<String>,
) -> anyhow::Result<Config> {
    let mut config = Config::default();
    let mut types = BTreeMap::new();
    let query = query.unwrap_or("Query".to_string());

    for file_descriptor in descriptor_set.file {
        append_enums(&mut types, file_descriptor.enum_type);
        append_msg_type(&mut types, file_descriptor.message_type);
        append_service(&mut types, file_descriptor.service, query.clone());
    }

    config.types = types;

    Ok(config)
}

#[cfg(test)]
mod test { // TODO add proper tests
    use prost_reflect::prost_types::FileDescriptorSet;

    use crate::config::from_proto::from_proto;

    static FOO: &str = r#"
    syntax = "proto3";

        import "google/protobuf/empty.proto";

        package news;

        enum Status {
          PUBLISHED = 0;
          DRAFT = 1;
        }


        message News {
          int32 id = 1;
          string title = 2;
          string body = 3;
          string postImage = 4;
          Status foo = 5;
        }

        service NewsService {
          rpc GetAllNews (google.protobuf.Empty) returns (NewsList) {}
          rpc GetNews (NewsId) returns (News) {}
          rpc GetMultipleNews (MultipleNewsId) returns (NewsList) {}
          rpc DeleteNews (NewsId) returns (google.protobuf.Empty) {}
          rpc EditNews (News) returns (News) {}
          rpc AddNews (News) returns (News) {}
        }

        message NewsId {
          int32 id = 1;
        }

        message MultipleNewsId {
          repeated NewsId ids = 1;
        }

        message NewsList {
          repeated News news = 1;
        }
    "#;

    #[test]
    fn test_from_proto() -> anyhow::Result<()> {
        let mut set = FileDescriptorSet::default();
        let file_desc = protox_parse::parse("news.proto", FOO)?;
        set.file.push(file_desc);

        let result = from_proto(set, None)?;
        println!("{}", result.to_sdl());
        Ok(())
    }
}
