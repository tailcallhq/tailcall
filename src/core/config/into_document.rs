use async_graphql::parser::types::*;
use async_graphql::{Pos, Positioned};
use async_graphql_value::{ConstValue, Name};

use super::{config, ConfigModule};
use crate::core::blueprint::TypeLike;
use crate::core::directive::DirectiveCodec;

fn pos<A>(a: A) -> Positioned<A> {
    Positioned::new(a, Pos::default())
}

fn convert_field(name: String, field: &config::Field) -> Positioned<FieldDefinition> {
    let directives = get_directives(field);
    let base_type = if field.list {
        async_graphql::parser::types::BaseType::List(Box::new(Type {
            nullable: !field.list_type_required,
            base: async_graphql::parser::types::BaseType::Named(Name::new(field.type_of.clone())),
        }))
    } else {
        async_graphql::parser::types::BaseType::Named(Name::new(field.type_of.clone()))
    };

    let args_map = field.args.clone();
    let args = args_map
        .iter()
        .map(|(name, arg)| {
            let base_type = if arg.list {
                async_graphql::parser::types::BaseType::List(Box::new(Type {
                    nullable: !arg.list_type_required(),
                    base: async_graphql::parser::types::BaseType::Named(Name::new(
                        arg.type_of.clone(),
                    )),
                }))
            } else {
                async_graphql::parser::types::BaseType::Named(Name::new(arg.type_of.clone()))
            };
            pos(async_graphql::parser::types::InputValueDefinition {
                description: arg.doc.clone().map(pos),
                name: pos(Name::new(name.clone())),
                ty: pos(Type { nullable: !arg.required, base: base_type }),

                default_value: arg
                    .default_value
                    .clone()
                    .map(|v| pos(ConstValue::String(v.to_string()))),
                directives: Vec::new(),
            })
        })
        .collect::<Vec<Positioned<InputValueDefinition>>>();

    pos(async_graphql::parser::types::FieldDefinition {
        description: field.doc.clone().map(pos),
        name: pos(Name::new(name)),
        arguments: args,
        ty: pos(Type { nullable: !field.required, base: base_type }),

        directives,
    })
}

struct ArgumentsConverter<'config> {
    config: &'config ConfigModule,
    output: Vec<Positioned<FieldDefinition>>,
}

impl<'config> ArgumentsConverter<'config> {
    fn new(config: &'config ConfigModule) -> Self {
        Self { config, output: Default::default() }
    }

    fn walk_arguments(
        &mut self,
        args: &[(&String, &config::Arg)],
        name: String,
        current_field: &mut config::Field,
    ) {
        let Some(&(arg_name, arg)) = args.first() else {
            self.output.push(convert_field(name, current_field));
            return;
        };

        let args = &args[1..];

        if let Some(union_) = self.config.find_union(&arg.type_of) {
            // if the type is union walk over all type members and generate new separate
            // field for this variant
            for (i, type_) in union_.types.iter().enumerate() {
                let new_arg = config::Arg { type_of: type_.clone(), ..arg.clone() };

                current_field.args.insert(arg_name.to_string(), new_arg);
                self.walk_arguments(args, format!("{name}Var{i}"), current_field);
            }
        } else {
            self.walk_arguments(args, name, current_field);
        }
    }

    fn arguments_to_fields(
        &mut self,
        name: String,
        field: &config::Field,
    ) -> Vec<Positioned<FieldDefinition>> {
        self.output = Vec::with_capacity(field.args.len());
        let args: Vec<_> = field.args.iter().collect();

        self.walk_arguments(&args, name, &mut field.clone());

        std::mem::take(&mut self.output)
    }
}

fn config_document(config: &ConfigModule) -> ServiceDocument {
    let mut definitions = Vec::new();
    let mut directives = vec![
        pos(config.server.to_directive()),
        pos(config.upstream.to_directive()),
    ];

    directives.extend(config.links.iter().map(|link| {
        let mut directive = link.to_directive();

        let type_directive = (
            pos(Name::new("type")),
            pos(ConstValue::Enum(Name::new(link.type_of.to_string()))),
        );

        directive.arguments = directive
            .arguments
            .iter()
            // "type" needs to be filtered out, because when is the default value, it is not present
            // in the directive
            .filter(|(name, _)| name != &pos(Name::new("type")))
            .map(|argument| argument.to_owned())
            .chain(std::iter::once(type_directive))
            .collect();

        pos(directive)
    }));

    let schema_definition = SchemaDefinition {
        extend: false,
        directives,
        query: config.schema.query.clone().map(|name| pos(Name::new(name))),
        mutation: config
            .schema
            .mutation
            .clone()
            .map(|name| pos(Name::new(name))),
        subscription: config
            .schema
            .subscription
            .clone()
            .map(|name| pos(Name::new(name))),
    };
    definitions.push(TypeSystemDefinition::Schema(pos(schema_definition)));
    for (type_name, type_def) in config.types.iter() {
        let kind = if config.interface_types.contains(type_name) {
            TypeKind::Interface(InterfaceType {
                implements: type_def
                    .implements
                    .iter()
                    .map(|name| pos(Name::new(name.clone())))
                    .collect(),
                fields: type_def
                    .fields
                    .clone()
                    .iter()
                    .map(|(name, field)| {
                        let directives = get_directives(field);
                        let base_type = if field.list {
                            BaseType::List(Box::new(Type {
                                nullable: !field.list_type_required,
                                base: BaseType::Named(Name::new(field.type_of.clone())),
                            }))
                        } else {
                            BaseType::Named(Name::new(field.type_of.clone()))
                        };
                        pos(FieldDefinition {
                            description: field.doc.clone().map(pos),
                            name: pos(Name::new(name.clone())),
                            arguments: vec![],
                            ty: pos(Type { nullable: !field.required, base: base_type }),

                            directives,
                        })
                    })
                    .collect::<Vec<Positioned<FieldDefinition>>>(),
            })
        } else if config.input_types.contains(type_name) {
            TypeKind::InputObject(InputObjectType {
                fields: type_def
                    .fields
                    .clone()
                    .iter()
                    .map(|(name, field)| {
                        let directives = get_directives(field);
                        let base_type = if field.list {
                            async_graphql::parser::types::BaseType::List(Box::new(Type {
                                nullable: !field.list_type_required,
                                base: async_graphql::parser::types::BaseType::Named(Name::new(
                                    field.type_of.clone(),
                                )),
                            }))
                        } else {
                            async_graphql::parser::types::BaseType::Named(Name::new(
                                field.type_of.clone(),
                            ))
                        };

                        pos(async_graphql::parser::types::InputValueDefinition {
                            description: field.doc.clone().map(pos),
                            name: pos(Name::new(name.clone())),
                            ty: pos(Type { nullable: !field.required, base: base_type }),

                            default_value: None,
                            directives,
                        })
                    })
                    .collect::<Vec<Positioned<InputValueDefinition>>>(),
            })
        } else if type_def.fields.is_empty() {
            TypeKind::Scalar
        } else {
            TypeKind::Object(ObjectType {
                implements: type_def
                    .implements
                    .iter()
                    .map(|name| pos(Name::new(name.clone())))
                    .collect(),
                fields: type_def
                    .fields
                    .iter()
                    .flat_map(|(name, field)| {
                        let mut converter = ArgumentsConverter::new(config);

                        converter.arguments_to_fields(name.clone(), field)
                    })
                    .collect::<Vec<Positioned<FieldDefinition>>>(),
            })
        };
        definitions.push(TypeSystemDefinition::Type(pos(TypeDefinition {
            extend: false,
            description: None,
            name: pos(Name::new(type_name.clone())),
            directives: type_def
                .added_fields
                .iter()
                .map(|added_field: &super::AddField| pos(added_field.to_directive()))
                .chain(
                    type_def
                        .cache
                        .as_ref()
                        .map(|cache| pos(cache.to_directive())),
                )
                .chain(
                    type_def
                        .protected
                        .as_ref()
                        .map(|protected| pos(protected.to_directive())),
                )
                .chain(type_def.tag.as_ref().map(|tag| pos(tag.to_directive())))
                .collect::<Vec<_>>(),
            kind,
        })));
    }
    for (name, union) in config.unions.iter() {
        // unions are only supported as the output types
        if !config.input_types.contains(name) {
            definitions.push(TypeSystemDefinition::Type(pos(TypeDefinition {
                extend: false,
                description: None,
                name: pos(Name::new(name)),
                directives: Vec::new(),
                kind: TypeKind::Union(UnionType {
                    members: union
                        .types
                        .iter()
                        .map(|name| pos(Name::new(name.clone())))
                        .collect(),
                }),
            })));
        }
    }

    for (name, values) in config.enums.iter() {
        definitions.push(TypeSystemDefinition::Type(pos(TypeDefinition {
            extend: false,
            description: values.doc.clone().map(pos),
            name: pos(Name::new(name)),
            directives: Vec::new(),
            kind: TypeKind::Enum(EnumType {
                values: values
                    .variants
                    .iter()
                    .map(|variant| {
                        pos(EnumValueDefinition {
                            description: None,
                            value: pos(Name::new(variant)),
                            directives: Vec::new(),
                        })
                    })
                    .collect(),
            }),
        })));
    }

    ServiceDocument { definitions }
}

fn get_directives(field: &crate::core::config::Field) -> Vec<Positioned<ConstDirective>> {
    let directives = vec![
        field.http.as_ref().map(|d| pos(d.to_directive())),
        field.script.as_ref().map(|d| pos(d.to_directive())),
        field.const_field.as_ref().map(|d| pos(d.to_directive())),
        field.modify.as_ref().map(|d| pos(d.to_directive())),
        field.omit.as_ref().map(|d| pos(d.to_directive())),
        field.graphql.as_ref().map(|d| pos(d.to_directive())),
        field.grpc.as_ref().map(|d| pos(d.to_directive())),
        field.cache.as_ref().map(|d| pos(d.to_directive())),
        field.call.as_ref().map(|d| pos(d.to_directive())),
        field.protected.as_ref().map(|d| pos(d.to_directive())),
    ];

    directives.into_iter().flatten().collect()
}

impl From<ConfigModule> for ServiceDocument {
    fn from(value: ConfigModule) -> Self {
        config_document(&value)
    }
}
