use std::collections::{BTreeSet, HashSet};

use anyhow::{bail, Result};
use derive_setters::Setters;
use prost_reflect::prost_types::field_descriptor_proto::Label;
use prost_reflect::prost_types::{
    DescriptorProto, EnumDescriptorProto, FileDescriptorSet, ServiceDescriptorProto, SourceCodeInfo,
};

use super::graphql_type::{GraphQLType, Unparsed};
use super::proto::comments_builder::CommentsBuilder;
use super::proto::path_builder::PathBuilder;
use super::proto::path_field::PathField;
use crate::core::config::transformer::{AmbiguousType, TreeShake};
use crate::core::config::{Arg, Config, Enum, Field, Grpc, Tag, Type, Union, Variant};
use crate::core::transform::{Transform, TransformerOps};
use crate::core::valid::Validator;

/// Assists in the mapping and retrieval of proto type names to custom formatted
/// strings based on the descriptor type.
#[derive(Setters)]
struct Context {
    /// The current proto package name.
    namespace: Vec<String>,

    /// Final configuration that's being built up.
    config: Config,

    /// Root GraphQL query type
    query: String,

    /// Set of visited map types
    map_types: HashSet<String>,

    /// Optional field to store source code information, including comments, for
    /// each entity.
    comments_builder: CommentsBuilder,
}

impl Context {
    fn new(query: &str) -> Self {
        Self {
            query: query.to_string(),
            namespace: Default::default(),
            config: Default::default(),
            map_types: Default::default(),
            comments_builder: CommentsBuilder::new(None),
        }
    }

    /// Sets source code information for preservation of comments.
    fn with_source_code_info(mut self, source_code_info: SourceCodeInfo) -> Self {
        self.comments_builder = CommentsBuilder::new(Some(source_code_info));
        self
    }

    /// Resolves the actual name and inserts the type.
    fn insert_type(mut self, name: String, ty: Type) -> Self {
        self.config.types.insert(name.to_string(), ty);
        self
    }

    /// Converts oneof definitions in message to set of types with union
    fn insert_oneofs(
        mut self,
        type_name: String, // name of the message
        base_type: Type,   // that's the type with fields that are not oneofs
        oneof_fields: Vec<Vec<(String, Field)>>, /* there is multiple oneof definitions every
                            * one of which contains multiple fields */
    ) -> Self {
        fn collect_types(
            type_name: String,
            base_type: Type,
            oneof_fields: &[Vec<(String, Field)>], // currently processed set of oneof fields
            output: &mut Vec<(String, Type)>,      // newly generated types with their names
        ) {
            let Some(one_of) = oneof_fields.first() else {
                output.push((type_name, base_type));

                return;
            };
            let oneof_fields = &oneof_fields[1..];

            // is there is only one variant for oneof field
            // this field is actually an optional field
            // for graphql and we can generate only single variant for this
            if one_of.len() == 1 {
                let (field_name, field) = one_of[0].clone();
                let mut new_type = base_type;
                new_type.fields.insert(field_name, field);

                collect_types(format!("{type_name}__Var"), new_type, oneof_fields, output);

                return;
            }

            // since all of the fields are actually optional in protobuf generate also a
            // type without this oneof completely
            collect_types(
                format!("{type_name}__Var"),
                base_type.clone(),
                oneof_fields,
                output,
            );

            for (i, (field_name, field)) in one_of.iter().enumerate() {
                let mut new_type = base_type.clone();
                let mut field = field.clone();

                // mark this field as required to force type-check on specific variant of oneof
                field.required = true;

                // add new field specific to this variant of oneof field
                new_type.fields.insert(field_name.clone(), field);

                collect_types(
                    format!("{type_name}__Var{i}"),
                    new_type,
                    oneof_fields,
                    output,
                );
            }
        }

        let mut union_types = Vec::new();

        collect_types(
            type_name.clone(),
            base_type,
            &oneof_fields,
            &mut union_types,
        );

        // if there is only one type in union no need
        // to actually create union and use just this type
        if union_types.len() == 1 {
            let (_, ty) = union_types.pop().unwrap();
            self.config.types.insert(type_name, ty);
            return self;
        }

        let mut union_ = Union::default();

        for (type_name, ty) in union_types {
            union_.types.insert(type_name.clone());

            self = self.insert_type(type_name, ty);
        }

        self.config.unions.insert(type_name, union_);

        self
    }

    /// Processes proto enum types.
    fn append_enums(
        mut self,
        enums: &[EnumDescriptorProto],
        parent_path: &PathBuilder,
        is_nested: bool,
    ) -> Self {
        for (index, enum_) in enums.iter().enumerate() {
            let enum_name = enum_.name();

            let enum_type_path = if is_nested {
                parent_path.extend(PathField::NestedEnum, index as i32)
            } else {
                parent_path.extend(PathField::EnumType, index as i32)
            };

            let mut variants_with_comments = BTreeSet::new();

            for (value_index, v) in enum_.value.iter().enumerate() {
                let variant_name = GraphQLType::new(v.name()).into_enum_variant().to_string();

                // Path to the enum value's comments
                let value_path = PathBuilder::new(&enum_type_path)
                    .extend(PathField::EnumValue, value_index as i32); // 2: value field

                // Get comments for the enum value
                let comment = self.comments_builder.get_comments(&value_path);

                // Format the variant with its comment as description
                if let Some(comment) = comment {
                    // TODO: better support for enum variant descriptions [There is no way to define
                    // description for enum variant in current config structure]
                    let variant_with_comment =
                        format!("\"\"\n  {}\n  \"\"\n  {}", comment, variant_name);
                    variants_with_comments.insert(variant_with_comment);
                } else {
                    variants_with_comments.insert(variant_name);
                }
            }

            let type_name = GraphQLType::new(enum_name)
                .extend(self.namespace.as_slice())
                .into_enum()
                .to_string();

            let doc = self.comments_builder.get_comments(&enum_type_path);

            let variants_with_comments = variants_with_comments
                .into_iter()
                .map(|v| Variant { name: v, alias: None })
                .collect();

            self.config
                .enums
                .insert(type_name, Enum { variants: variants_with_comments, doc });
        }
        self
    }

    /// Processes proto message types.
    fn append_msg_type(
        mut self,
        messages: &[DescriptorProto],
        parent_path: &PathBuilder,
        is_nested: bool,
    ) -> Result<Self> {
        for (index, message) in messages.iter().enumerate() {
            let msg_name = message.name();

            let msg_type = GraphQLType::new(msg_name)
                .extend(self.namespace.as_slice())
                .into_object_type();

            if message
                .options
                .as_ref()
                .and_then(|opt| opt.map_entry)
                .unwrap_or_default()
            {
                // map types in protobuf are encoded as nested type
                // https://protobuf.dev/programming-guides/encoding/#maps
                // since we encode it as JSON scalar type in graphQL
                // record that this type is map and ignore it
                self.map_types.insert(msg_type.id());
                continue;
            }

            let msg_path = if is_nested {
                parent_path.extend(PathField::NestedType, index as i32)
            } else {
                parent_path.extend(PathField::MessageType, index as i32)
            };

            // first append the name of current message as namespace
            self.namespace.push(msg_name.to_string());
            self = self.append_enums(&message.enum_type, &PathBuilder::new(&msg_path), true);
            self =
                self.append_msg_type(&message.nested_type, &PathBuilder::new(&msg_path), true)?;
            // then drop it after handling nested types
            self.namespace.pop();

            let mut oneof_fields: Vec<_> = message.oneof_decl.iter().map(|_| Vec::new()).collect();

            let mut ty = Type {
                doc: self.comments_builder.get_comments(&msg_path),
                ..Default::default()
            };

            for (field_index, field) in message.field.iter().enumerate() {
                let field_name = GraphQLType::new(field.name())
                    .extend(self.namespace.as_slice())
                    .into_field();

                let mut cfg_field = Field::default();

                let label = field.label();
                cfg_field.list = matches!(label, Label::Repeated);
                // required only applicable for proto2
                cfg_field.required = matches!(label, Label::Required);

                if let Some(type_name) = &field.type_name {
                    // check that current field is map.
                    // it's done by checking that we've seen this type before
                    // inside the nested type. It works only if we explore nested types
                    // before the current type
                    if self.map_types.contains(&type_name[1..]) {
                        cfg_field.type_of = "JSON".to_string();
                        // drop list option since it is not relevant
                        // when using JSON representation
                        cfg_field.list = false;
                    } else {
                        // for non-primitive types
                        let type_of = graphql_type_from_ref(type_name)?
                            .into_object_type()
                            .to_string();

                        cfg_field.type_of = type_of;
                    }
                } else {
                    let type_of = convert_primitive_type(field.r#type().as_str_name());

                    cfg_field.type_of = type_of;
                }

                let field_path =
                    PathBuilder::new(&msg_path).extend(PathField::Field, field_index as i32);
                cfg_field.doc = self.comments_builder.get_comments(&field_path);

                if let Some(oneof_index) = field.oneof_index {
                    oneof_fields[oneof_index as usize].push((field_name.to_string(), cfg_field));
                } else {
                    ty.fields.insert(field_name.to_string(), cfg_field);
                }
            }

            ty.tag = Some(Tag { id: msg_type.id() });

            if message.oneof_decl.is_empty() {
                self = self.insert_type(msg_type.to_string(), ty);
            } else {
                self = self.insert_oneofs(msg_type.to_string(), ty, oneof_fields);
            }
        }
        Ok(self)
    }

    /// Processes proto service definitions and their methods.
    fn append_query_service(
        mut self,
        services: &[ServiceDescriptorProto],
        parent_path: &PathBuilder,
    ) -> Result<Self> {
        if services.is_empty() {
            return Ok(self);
        }

        for (index, service) in services.iter().enumerate() {
            let service_name = service.name();
            let path = parent_path.extend(PathField::Service, index as i32);

            for (method_index, method) in service.method.iter().enumerate() {
                let field_name = GraphQLType::new(method.name())
                    .extend(self.namespace.as_slice())
                    .push(service_name)
                    .into_method();

                let mut cfg_field = Field::default();
                let mut body = None;

                if let Some(graphql_type) = get_input_type(method.input_type())? {
                    let key = graphql_type.clone().into_field().to_string();
                    let type_of = graphql_type.into_object_type().to_string();
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

                    body = Some(format!("{{{{.args.{key}}}}}"));
                    cfg_field.args.insert(key, val);
                }

                let output_ty = get_output_type(method.output_type())?
                    .into_object_type()
                    .to_string();
                cfg_field.type_of = output_ty;
                cfg_field.required = true;

                cfg_field.grpc = Some(Grpc {
                    base_url: None,
                    body,
                    group_by: vec![],
                    headers: vec![],
                    method: field_name.id(),
                });

                let method_path =
                    PathBuilder::new(&path).extend(PathField::Method, method_index as i32);
                cfg_field.doc = self.comments_builder.get_comments(&method_path);

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
        Ok(self)
    }
}

fn graphql_type_from_ref(name: &str) -> Result<GraphQLType<Unparsed>> {
    if !name.starts_with('.') {
        bail!("Expected fully-qualified name for reference type but got {name}. This is a bug!");
    }

    let name = &name[1..];

    if let Some((package, name)) = name.rsplit_once('.') {
        Ok(GraphQLType::new(name).push(package))
    } else {
        Ok(GraphQLType::new(name))
    }
}

/// Converts proto field types to a custom format.
fn convert_primitive_type(proto_ty: &str) -> String {
    let binding = proto_ty.to_lowercase();
    let proto_ty = binding.strip_prefix("type_").unwrap_or(proto_ty);
    // use Int64Str and Uint64Str to represent 64bit integers as string by default
    // it's how this values are represented in JSON by default in prost
    // see tests in `protobuf::tests::scalars_proto_file`
    match proto_ty {
        "double" | "float" => "Float",
        "int32" | "sint32" | "fixed32" | "sfixed32" => "Int",
        "int64" | "sint64" | "fixed64" | "sfixed64" => "Int64",
        "uint32" => "UInt32",
        "uint64" => "UInt64",
        "bool" => "Boolean",
        "string" => "String",
        "bytes" => "Bytes",
        x => x,
    }
    .to_string()
}

/// Determines the output type for a service method.
fn get_output_type(output_ty: &str) -> Result<GraphQLType<Unparsed>> {
    // type, required
    match output_ty {
        ".google.protobuf.Empty" => {
            // If it's no response is expected, we return an Empty scalar type
            Ok(GraphQLType::new("Empty"))
        }
        _ => {
            // Setting it not null by default. There's no way to infer this from proto file
            graphql_type_from_ref(output_ty)
        }
    }
}

fn get_input_type(input_ty: &str) -> Result<Option<GraphQLType<Unparsed>>> {
    match input_ty {
        ".google.protobuf.Empty" | "" => Ok(None),
        _ => graphql_type_from_ref(input_ty).map(Some),
    }
}

/// The main entry point that builds a Config object from proto descriptor sets.
pub fn from_proto(descriptor_sets: &[FileDescriptorSet], query: &str) -> Result<Config> {
    let mut ctx = Context::new(query);
    for descriptor_set in descriptor_sets.iter() {
        for file_descriptor in descriptor_set.file.iter() {
            ctx.namespace = vec![file_descriptor.package().to_string()];

            if let Some(source_code_info) = &file_descriptor.source_code_info {
                ctx = ctx.with_source_code_info(source_code_info.clone());
            }

            let root_path = PathBuilder::new(&[]);

            ctx = ctx
                .append_enums(&file_descriptor.enum_type, &root_path, false)
                .append_msg_type(&file_descriptor.message_type, &root_path, false)?
                .append_query_service(&file_descriptor.service, &root_path)?;
        }
    }

    let config = AmbiguousType::default()
        .pipe(TreeShake)
        .transform(ctx.config)
        .to_result()?;

    Ok(config)
}

#[cfg(test)]
mod test {
    use anyhow::Result;
    use prost_reflect::prost_types::FileDescriptorSet;
    use tailcall_fixtures::protobuf;

    use super::from_proto;
    use crate::core::config::ConfigModule;

    fn compile_protobuf(files: &[&str]) -> Result<FileDescriptorSet> {
        Ok(protox::compile(files, [protobuf::SELF])?)
    }

    macro_rules! assert_gen {
        ($( $set:expr ), +) => {
            let set = compile_protobuf(&[$( $set ),+]).unwrap();
            let config = from_proto(&[set], "Query").unwrap();
            let config_module = ConfigModule::from(config);
            let result = config_module.to_sdl();
            insta::assert_snapshot!(result);
        };
    }

    #[test]
    fn test_from_proto() {
        // news_enum.proto covers:
        // test for mutation
        // test for empty objects
        // test for optional type
        // test for enum
        // test for repeated fields
        // test for a type used as both input and output
        // test for two types having same name in different packages

        assert_gen!(protobuf::NEWS, protobuf::GREETINGS_A, protobuf::GREETINGS_B);
    }

    #[test]
    fn test_from_proto_no_pkg_file() {
        assert_gen!(protobuf::NEWS_NO_PKG);
    }

    #[test]
    fn test_from_proto_no_service_file() {
        assert_gen!(protobuf::NEWS_NO_SERVICE);
    }

    #[test]
    fn test_greetings_proto_file() {
        assert_gen!(protobuf::GREETINGS, protobuf::GREETINGS_MESSAGE);
    }

    #[test]
    fn test_config_from_sdl() -> Result<()> {
        let set =
            compile_protobuf(&[protobuf::NEWS, protobuf::GREETINGS_A, protobuf::GREETINGS_B])?;

        let set1 = compile_protobuf(&[protobuf::NEWS])?;
        let set2 = compile_protobuf(&[protobuf::GREETINGS_A])?;
        let set3 = compile_protobuf(&[protobuf::GREETINGS_B])?;

        let actual = from_proto(&[set.clone()], "Query")?.to_sdl();
        let expected = from_proto(&[set1, set2, set3], "Query")?.to_sdl();

        pretty_assertions::assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_required_types() {
        // required fields are deprecated in proto3 (https://protobuf.dev/programming-guides/dos-donts/#add-required)
        // this example uses proto2 to test the same.
        // for proto3 it's guaranteed to have a default value (https://protobuf.dev/programming-guides/proto3/#default)
        // and our implementation (https://github.com/tailcallhq/tailcall/pull/1537) supports default values by default.
        // so we do not need to explicitly mark fields as required.

        assert_gen!(protobuf::PERSON);
    }

    #[test]
    fn test_movies() {
        assert_gen!(protobuf::MOVIES);
    }

    #[test]
    fn test_nested_types() {
        assert_gen!(protobuf::NESTED_TYPES);
    }

    #[test]
    fn test_map_types() {
        assert_gen!(protobuf::MAP);
    }

    #[test]
    fn test_optional_fields() {
        assert_gen!(protobuf::OPTIONAL);
    }

    #[test]
    fn test_scalar_types() {
        assert_gen!(protobuf::SCALARS);
    }

    #[test]
    fn test_oneof_types() {
        assert_gen!(protobuf::ONEOF);
    }
}
