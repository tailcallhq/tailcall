use async_graphql::parser::types::*;
use async_graphql::Positioned;
use async_graphql_value::{ConstValue, Name};
use tailcall_valid::Validator;

use super::directive::to_const_directive;
use super::Config;
use crate::core::directive::DirectiveCodec;
use crate::core::pos;

fn transform_default_value(value: Option<serde_json::Value>) -> Option<ConstValue> {
    value.map(ConstValue::from_json).and_then(Result::ok)
}

fn config_document(config: &Config) -> ServiceDocument {
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
    let interface_types = config.interfaces_types_map();
    let input_types = config.input_types();
    for (type_name, type_def) in config.types.iter() {
        let kind = if interface_types.contains_key(type_name) {
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
                        let type_of = &field.type_of;
                        let directives = field_directives(field);
                        pos(FieldDefinition {
                            description: field.doc.clone().map(pos),
                            name: pos(Name::new(name.clone())),
                            arguments: vec![],
                            ty: pos(type_of.into()),
                            directives,
                        })
                    })
                    .collect::<Vec<Positioned<FieldDefinition>>>(),
            })
        } else if input_types.contains(type_name) {
            TypeKind::InputObject(InputObjectType {
                fields: type_def
                    .fields
                    .iter()
                    .map(|(name, field)| {
                        let type_of = &field.type_of;
                        let directives = field_directives(field);

                        pos(async_graphql::parser::types::InputValueDefinition {
                            description: field.doc.clone().map(pos),
                            name: pos(Name::new(name.clone())),
                            ty: pos(type_of.into()),
                            default_value: transform_default_value(field.default_value.clone())
                                .map(pos),
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
                    .map(|(name, field)| {
                        let type_of = &field.type_of;
                        let directives = field_directives(field);

                        let args_map = field.args.clone();
                        let args = args_map
                            .iter()
                            .map(|(name, arg)| {
                                pos(async_graphql::parser::types::InputValueDefinition {
                                    description: arg.doc.clone().map(pos),
                                    name: pos(Name::new(name.clone())),
                                    ty: pos((&arg.type_of).into()),

                                    default_value: transform_default_value(
                                        arg.default_value.clone(),
                                    )
                                    .map(pos),
                                    directives: Vec::new(),
                                })
                            })
                            .collect::<Vec<Positioned<InputValueDefinition>>>();

                        pos(async_graphql::parser::types::FieldDefinition {
                            description: field.doc.clone().map(pos),
                            name: pos(Name::new(name.clone())),
                            arguments: args,
                            ty: pos(type_of.into()),
                            directives,
                        })
                    })
                    .collect::<Vec<Positioned<FieldDefinition>>>(),
            })
        };

        let directives = type_directives(type_def);

        definitions.push(TypeSystemDefinition::Type(pos(TypeDefinition {
            extend: false,
            description: type_def.doc.clone().map(pos),
            name: pos(Name::new(type_name.clone())),
            directives,
            kind,
        })));
    }
    for (name, union) in config.unions.iter() {
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
                            value: pos(Name::new(&variant.name)),
                            directives: variant
                                .alias
                                .clone()
                                .map_or(vec![], |v| vec![pos(v.to_directive())]),
                        })
                    })
                    .collect(),
            }),
        })));
    }

    ServiceDocument { definitions }
}

fn into_directives(
    directives: &[super::directive::Directive],
) -> impl Iterator<Item = Positioned<ConstDirective>> + '_ {
    directives
        .iter()
        .filter_map(|d| to_const_directive(d).to_result().ok())
        .map(pos)
}

fn field_directives(field: &crate::core::config::Field) -> Vec<Positioned<ConstDirective>> {
    field
        .resolvers
        .iter()
        .filter_map(|resolver| resolver.to_directive().map(pos))
        .chain(field.modify.as_ref().map(|d| pos(d.to_directive())))
        .chain(field.omit.as_ref().map(|d| pos(d.to_directive())))
        .chain(field.cache.as_ref().map(|d| pos(d.to_directive())))
        .chain(field.protected.as_ref().map(|d| pos(d.to_directive())))
        .chain(into_directives(&field.directives))
        .collect()
}

fn type_directives(type_def: &crate::core::config::Type) -> Vec<Positioned<ConstDirective>> {
    type_def
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
        .chain(
            type_def
                .resolvers
                .iter()
                .filter_map(|resolver| resolver.to_directive().map(pos)),
        )
        .chain(into_directives(&type_def.directives))
        .collect::<Vec<_>>()
}

impl From<&Config> for ServiceDocument {
    fn from(value: &Config) -> Self {
        config_document(value)
    }
}
